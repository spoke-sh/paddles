{
  description = "paddles development environment";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    rust-overlay.url = "github:oxalica/rust-overlay";
    flake-utils.url = "github:numtide/flake-utils";

    sift = {
      url = "github:rupurt/sift?rev=003932b6e57c8fc1e81950b1066afe4feec66c0d";
      inputs.keel.follows = "keel";
    };

    keel = {
      url = "github:spoke-sh/keel?ref=main";
      inputs.nixpkgs.follows = "nixpkgs";
      inputs.rust-overlay.follows = "rust-overlay";
      inputs.flake-utils.follows = "flake-utils";
    };
  };

  outputs = { nixpkgs, rust-overlay, flake-utils, keel, sift, ... }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs {
          inherit system overlays;
          config.allowUnfree = true;
        };
        rust = pkgs.rust-bin.stable.latest.default.override {
          extensions = [ "rust-src" "rust-analyzer" "llvm-tools" ];
        };
        # Keep package builds on the same overlaid toolchain as the repo/dev shell.
        # Edition 2024 dependencies can outpace nixpkgs' default rustPlatform cargo.
        rustPlatform = pkgs.makeRustPlatform {
          cargo = rust;
          rustc = rust;
        };
        isLinux = pkgs.stdenv.isLinux;
        isDarwin = pkgs.stdenv.isDarwin;
        cudaToolkit = pkgs.cudatoolkit;

        # Build sift directly from the pinned source so paddles can keep
        # a CPU-first default even before upstream sift revisions are updated.
        siftSrc = sift.outPath;
        siftVersion = (builtins.fromTOML (builtins.readFile "${siftSrc}/Cargo.toml")).package.version;
        buildSiftPackage = rustPlatform: pname: cudaEnabled:
          rustPlatform.buildRustPackage {
            inherit pname;
            version = siftVersion;
            src = siftSrc;
            cargoLock = {
              lockFile = "${siftSrc}/Cargo.lock";
              outputHashes = {
                "candle-core-0.9.2" = "sha256-Oa62yRA95P/MsGUG2u10a/jgcRtUdVFOQIoykqmv4Bs=";
                "candle-nn-0.9.2" = "sha256-Oa62yRA95P/MsGUG2u10a/jgcRtUdVFOQIoykqmv4Bs=";
                "candle-transformers-0.9.2" = "sha256-Oa62yRA95P/MsGUG2u10a/jgcRtUdVFOQIoykqmv4Bs=";
                "candle-kernels-0.9.2" = "sha256-Oa62yRA95P/MsGUG2u10a/jgcRtUdVFOQIoykqmv4Bs=";
                "candle-ug-0.9.2" = "sha256-Oa62yRA95P/MsGUG2u10a/jgcRtUdVFOQIoykqmv4Bs=";
                "metamorph-0.1.0" = "sha256-sGl4+khLHI2k4gX/jikg9ZcVDknQNKXYWHuV2uZtnCc=";
              };
            };
            nativeBuildInputs = [ pkgs.pkg-config ] ++ pkgs.lib.optionals (isLinux && cudaEnabled) [ cudaToolkit ];
            buildInputs = [ pkgs.bzip2 pkgs.xz pkgs.zlib ] ++ pkgs.lib.optionals (isLinux && cudaEnabled) [ cudaToolkit ];
            buildFeatures = pkgs.lib.optionals (isLinux && cudaEnabled) [ "cuda" ];
            doCheck = false;

            CUDA_HOME = pkgs.lib.optionalString (isLinux && cudaEnabled) "${cudaToolkit}";
            CUDA_PATH = pkgs.lib.optionalString (isLinux && cudaEnabled) "${cudaToolkit}";
            CUDA_ROOT = pkgs.lib.optionalString (isLinux && cudaEnabled) "${cudaToolkit}";
            CUDA_TOOLKIT_ROOT_DIR = pkgs.lib.optionalString (isLinux && cudaEnabled) "${cudaToolkit}";
            CUDA_COMPUTE_CAP = pkgs.lib.optionalString (isLinux && cudaEnabled) "80";
            NVCC_PREPEND_FLAGS = pkgs.lib.optionalString (isLinux && cudaEnabled) "-I${cudaToolkit}/include";
          };

        siftPkg = buildSiftPackage rustPlatform "sift" false;
        siftCudaPkg = buildSiftPackage rustPlatform "sift-cuda" true;
        keelPkg = keel.packages.${system}.keel;

        cargoToml = (builtins.fromTOML (builtins.readFile ./Cargo.toml));
        version = cargoToml.package.version;
        oldMetamorphDependencyRef = "metamorph 0.1.0 (git+https://github.com/rupurt/metamorph?rev=00ac826973d378ce7670b1057bfd467f6cf8de29)";
        newMetamorphDependencyRef = "metamorph 0.1.0 (git+https://github.com/rupurt/metamorph?rev=509b6718aa07333f2c763b618cdcf4a5d43d27cc)";
        oldMetamorphReplacementEntry = builtins.concatStringsSep "\n" [
          "[[package]]"
          "name = \"metamorph\""
          "version = \"0.1.0\""
          "source = \"git+https://github.com/rupurt/metamorph?rev=00ac826973d378ce7670b1057bfd467f6cf8de29#00ac826973d378ce7670b1057bfd467f6cf8de29\""
          "replace = \"${newMetamorphDependencyRef}\""
          ""
          ""
        ];
        # nixpkgs vendors git crates by name/version, so normalize the lockfile
        # before importCargoLock collapses both metamorph revisions to one path.
        normalizedCargoLock = builtins.replaceStrings
          [
            oldMetamorphReplacementEntry
            oldMetamorphDependencyRef
          ]
          [
            ""
            newMetamorphDependencyRef
          ]
          (builtins.readFile ./Cargo.lock);
        normalizedCargoLockFile = pkgs.writeText "Cargo.lock" normalizedCargoLock;
        buildPaddlesPackage = pname: cudaEnabled:
          rustPlatform.buildRustPackage {
            inherit pname version;
            src = ./.;
            postPatch = ''
              cp ${normalizedCargoLockFile} "''${cargoRoot:+$cargoRoot/}Cargo.lock"
              substituteInPlace "$cargoDepsCopy/sift-0.2.0/Cargo.toml" \
                --replace-fail 'rev = "00ac826973d378ce7670b1057bfd467f6cf8de29"' 'rev = "509b6718aa07333f2c763b618cdcf4a5d43d27cc"'
            '';
            cargoLock = {
              lockFileContents = normalizedCargoLock;
              allowBuiltinFetchGit = true;
              outputHashes = {
                "candle-core-0.9.2" = "sha256-ywjfKjuViDvJEho/IO2jR73ObwbMznWwQgSrAbJS1v0=";
                "candle-nn-0.9.2" = "sha256-ywjfKjuViDvJEho/IO2jR73ObwbMznWwQgSrAbJS1v0=";
                "candle-transformers-0.9.2" = "sha256-ywjfKjuViDvJEho/IO2jR73ObwbMznWwQgSrAbJS1v0=";
                "candle-kernels-0.9.2" = "sha256-ywjfKjuViDvJEho/IO2jR73ObwbMznWwQgSrAbJS1v0=";
                "candle-ug-0.9.2" = "sha256-ywjfKjuViDvJEho/IO2jR73ObwbMznWwQgSrAbJS1v0=";
                "sift-0.2.0" = "sha256-mlOVOWZFB0vrWlCy/ntlj3LRcn8HiojN4Wv+K9hnQ5U=";
                "transit-core-0.1.0" = "sha256-KRsbmHsTZoH9AZTPIiIkozchXBfWZ1XK3rXkXnBsh1U=";
              };
            };
            nativeBuildInputs = [ pkgs.pkg-config ] ++ pkgs.lib.optionals (isLinux && cudaEnabled) [ cudaToolkit ];
            buildInputs = [ pkgs.openssl pkgs.zlib pkgs.zstd ] ++ pkgs.lib.optionals (isLinux && cudaEnabled) [ cudaToolkit ];
            buildFeatures = pkgs.lib.optionals (isLinux && cudaEnabled) [ "cuda" ];
            doCheck = false;

            CUDA_PATH = pkgs.lib.optionalString (isLinux && cudaEnabled) "${cudaToolkit}";
            CUDA_COMPUTE_CAP = pkgs.lib.optionalString (isLinux && cudaEnabled) "80";
            NVCC_PREPEND_FLAGS = pkgs.lib.optionalString (isLinux && cudaEnabled) "-I${cudaToolkit}/include";
          };
        baseShellInputs = [
          rust
          pkgs.nodejs_20
          pkgs.just
          pkgs.cargo-nextest
          pkgs.cargo-llvm-cov
          pkgs.pkg-config
          pkgs.openssl
          pkgs.zlib
          pkgs.zstd
          keelPkg
        ];
        linuxShellInputs = pkgs.lib.optionals isLinux [
          pkgs.chromium
          pkgs.mold
        ];
        defaultShellHook = ''
          # Shared target directory across shell sessions for faster rebuilds.
          export CARGO_TARGET_DIR="$HOME/.cache/cargo-target/paddles"
        ''
        + pkgs.lib.optionalString isLinux ''
          export PLAYWRIGHT_CHROMIUM_EXECUTABLE_PATH="${pkgs.lib.getExe pkgs.chromium}"
          export PLAYWRIGHT_SKIP_BROWSER_DOWNLOAD=1

          build_lib_path="${pkgs.lib.makeLibraryPath [ pkgs.zlib pkgs.zstd pkgs.openssl ]}"
          export LIBRARY_PATH="$build_lib_path''${LIBRARY_PATH:+:$LIBRARY_PATH}"

          linux_link_args="-C link-arg=-fuse-ld=mold"
          linux_link_args="$linux_link_args -C link-arg=-Wl,-rpath,${pkgs.lib.getLib pkgs.zlib}/lib"
          linux_link_args="$linux_link_args -C link-arg=-Wl,-rpath,${pkgs.lib.getLib pkgs.zstd}/lib"
          linux_link_args="$linux_link_args -C link-arg=-Wl,-rpath,${pkgs.lib.getLib pkgs.openssl}/lib"

          export CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_RUSTFLAGS="''${CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_RUSTFLAGS:+$CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_RUSTFLAGS }$linux_link_args"
          export CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_RUSTFLAGS="''${CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_RUSTFLAGS:+$CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_RUSTFLAGS }$linux_link_args"
        ''
        + pkgs.lib.optionalString isDarwin ''
          # Let Playwright manage its own browser download on macOS.
          # Nix can set TMPDIR to a shell-specific path on macOS; use a stable path.
          export TMPDIR=/var/tmp
        '';
        cudaShellHook = ''
          # Shared target directory across shell sessions for faster rebuilds.
          export CARGO_TARGET_DIR="$HOME/.cache/cargo-target/paddles"
          export PLAYWRIGHT_CHROMIUM_EXECUTABLE_PATH="${pkgs.lib.getExe pkgs.chromium}"
          export PLAYWRIGHT_SKIP_BROWSER_DOWNLOAD=1

          export CUDA_PATH="${cudaToolkit}"
          export CUDA_ROOT="${cudaToolkit}"
          export CUDA_TOOLKIT_ROOT_DIR="${cudaToolkit}"
          export CUDA_COMPUTE_CAP="''${CUDA_COMPUTE_CAP:-80}"

          build_lib_path="${pkgs.lib.makeLibraryPath [ pkgs.zlib pkgs.zstd pkgs.openssl ]}"
          for candidate in \
            "${cudaToolkit}/lib" \
            "${cudaToolkit}/lib64" \
            "${cudaToolkit}/targets/x86_64-linux/lib"
          do
            if [ -d "$candidate" ]; then
              build_lib_path="$candidate:$build_lib_path"
            fi
          done
          export LIBRARY_PATH="$build_lib_path''${LIBRARY_PATH:+:$LIBRARY_PATH}"

          linux_link_args="-C link-arg=-fuse-ld=mold"
          linux_link_args="$linux_link_args -C link-arg=-Wl,-rpath,${pkgs.lib.getLib pkgs.zlib}/lib"
          linux_link_args="$linux_link_args -C link-arg=-Wl,-rpath,${pkgs.lib.getLib pkgs.zstd}/lib"
          linux_link_args="$linux_link_args -C link-arg=-Wl,-rpath,${pkgs.lib.getLib pkgs.openssl}/lib"
          for candidate in \
            "${cudaToolkit}/lib" \
            "${cudaToolkit}/lib64" \
            "${cudaToolkit}/targets/x86_64-linux/lib"
          do
            if [ -d "$candidate" ]; then
              linux_link_args="$linux_link_args -C link-arg=-Wl,-rpath,$candidate"
            fi
          done

          for candidate in \
            /run/opengl-driver/lib \
            /usr/lib/x86_64-linux-gnu \
            /usr/lib/wsl/lib
          do
            if [ -f "$candidate/libcuda.so.1" ]; then
              linux_link_args="$linux_link_args -C link-arg=-Wl,-rpath,$candidate"
              break
            fi
          done

          export CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_RUSTFLAGS="''${CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_RUSTFLAGS:+$CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_RUSTFLAGS }$linux_link_args"
          export CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_RUSTFLAGS="''${CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_RUSTFLAGS:+$CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_RUSTFLAGS }$linux_link_args"
        '';

        paddlesPkg = buildPaddlesPackage "paddles" false;
        paddlesCudaPkg = buildPaddlesPackage "paddles-cuda" true;

      in {
        packages = {
          paddles = paddlesPkg;
          keel = keelPkg;
          sift = siftPkg;
          default = paddlesPkg;
        } // pkgs.lib.optionalAttrs isLinux {
          paddles-cuda = paddlesCudaPkg;
          sift-cuda = siftCudaPkg;
        };

        devShells = {
          default = pkgs.mkShell {
            buildInputs = baseShellInputs ++ [ siftPkg ] ++ linuxShellInputs;
            shellHook = defaultShellHook;
          };
        } // pkgs.lib.optionalAttrs isLinux {
          cuda = pkgs.mkShell {
            buildInputs = baseShellInputs ++ [ siftCudaPkg cudaToolkit ] ++ linuxShellInputs;
            shellHook = cudaShellHook;
          };
        };
      });
}
