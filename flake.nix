{
  description = "paddles development environment";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    rust-overlay.url = "github:oxalica/rust-overlay";
    flake-utils.url = "github:numtide/flake-utils";

    sift.url = "github:rupurt/sift?ref=main";

    keel = {
      url = "git+ssh://git@github.com/spoke-sh/keel.git?ref=main";
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
        isLinux = pkgs.stdenv.isLinux;
        isDarwin = pkgs.stdenv.isDarwin;

        siftPkg = sift.packages.${system}.sift;
        keelPkg = keel.packages.${system}.keel;

        cargoToml = (builtins.fromTOML (builtins.readFile ./Cargo.toml));
        version = cargoToml.package.version;

        paddlesPkg = pkgs.rustPlatform.buildRustPackage {
          pname = "paddles";
          inherit version;
          src = ./.;
          cargoLock = {
            lockFile = ./Cargo.lock;
            outputHashes = {
              "candle-core-0.9.2" = "sha256-GeU7yc4vqN0hy3tJAq0LDhwnpO4XDeVVmxaBchKWkWg=";
              "candle-nn-0.9.2" = "sha256-GeU7yc4vqN0hy3tJAq0LDhwnpO4XDeVVmxaBchKWkWg=";
              "candle-transformers-0.9.2" = "sha256-GeU7yc4vqN0hy3tJAq0LDhwnpO4XDeVVmxaBchKWkWg=";
              "candle-kernels-0.9.2" = "sha256-GeU7yc4vqN0hy3tJAq0LDhwnpO4XDeVVmxaBchKWkWg=";
              "candle-ug-0.9.2" = "sha256-GeU7yc4vqN0hy3tJAq0LDhwnpO4XDeVVmxaBchKWkWg=";
            };
          };
          nativeBuildInputs = [ pkgs.pkg-config ] ++ pkgs.lib.optionals isLinux [ pkgs.cudatoolkit ];
          buildInputs = [ pkgs.openssl pkgs.zlib pkgs.zstd ] ++ pkgs.lib.optionals isLinux [ pkgs.cudatoolkit ];
          doCheck = false;

          CUDA_PATH = pkgs.lib.optionalString isLinux "${pkgs.cudatoolkit}";
          CUDA_COMPUTE_CAP = "80";
          NVCC_PREPEND_FLAGS = pkgs.lib.optionalString isLinux "-I${pkgs.cudatoolkit}/include";
        };

      in {
        packages = {
          paddles = paddlesPkg;
          keel = keelPkg;
          sift = siftPkg;
          default = paddlesPkg;
        };

        devShells.default = pkgs.mkShell {
                    buildInputs = [
                      rust
                      pkgs.just
                      pkgs.cargo-nextest
                      pkgs.cargo-llvm-cov
                                  pkgs.pkg-config
                                              pkgs.openssl
                                              pkgs.zlib
                                              pkgs.zstd
                                              keelPkg
                                                        siftPkg
                    ]
           ++ pkgs.lib.optionals isLinux [
            pkgs.cudatoolkit
            pkgs.mold
          ];

                              shellHook = ''
                                # Shared target directory across shell sessions for faster rebuilds.
                                export CARGO_TARGET_DIR="$HOME/.cache/cargo-target/paddles"
                              ''

           + pkgs.lib.optionalString isDarwin ''
            # Nix can set TMPDIR to a shell-specific path on macOS; use a stable path.
            export TMPDIR=/var/tmp
          '' + pkgs.lib.optionalString isLinux ''
            # Expose the CUDA toolkit for Candle CUDA builds and runtime loading.
            export CUDA_PATH="${pkgs.cudatoolkit}"
            export CUDA_ROOT="${pkgs.cudatoolkit}"
            export CUDA_TOOLKIT_ROOT_DIR="${pkgs.cudatoolkit}"

            # Build-time library search path for the linker.
            # Unlike LD_LIBRARY_PATH, LIBRARY_PATH only affects the linker
            # and does not break non-nix binaries (nix, git, etc.).
            build_lib_path="${pkgs.lib.makeLibraryPath [ pkgs.zlib pkgs.zstd pkgs.openssl ]}"
            for candidate in \
              "${pkgs.cudatoolkit}/lib" \
              "${pkgs.cudatoolkit}/lib64" \
              "${pkgs.cudatoolkit}/targets/x86_64-linux/lib"
            do
              if [ -d "$candidate" ]; then
                build_lib_path="$candidate:$build_lib_path"
              fi
            done
            export LIBRARY_PATH="$build_lib_path''${LIBRARY_PATH:+:$LIBRARY_PATH}"

            # Bake rpath into compiled binaries so they find shared libs
            # at runtime without needing LD_LIBRARY_PATH.
            linux_link_args="-C link-arg=-fuse-ld=mold"
            linux_link_args="$linux_link_args -C link-arg=-Wl,-rpath,${pkgs.lib.getLib pkgs.zlib}/lib"
            linux_link_args="$linux_link_args -C link-arg=-Wl,-rpath,${pkgs.lib.getLib pkgs.zstd}/lib"
            linux_link_args="$linux_link_args -C link-arg=-Wl,-rpath,${pkgs.lib.getLib pkgs.openssl}/lib"
            for candidate in \
              "${pkgs.cudatoolkit}/lib" \
              "${pkgs.cudatoolkit}/lib64" \
              "${pkgs.cudatoolkit}/targets/x86_64-linux/lib"
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
            export CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_RUSTFLAGS="''${CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_RUSTFLAGS:+$CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_RUSTFLAGS }-C link-arg=-fuse-ld=mold"
          '';
        };
      });
}
