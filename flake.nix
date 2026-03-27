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
              "wonopcode-core-0.1.2" = "sha256-rlG+UKI9O4fu39GcCK03pRSbx8YmSjGVlZG6WIUcbuU=";
              "wonopcode-provider-0.1.2" = "sha256-rlG+UKI9O4fu39GcCK03pRSbx8YmSjGVlZG6WIUcbuU=";
              "wonopcode-tools-0.1.2" = "sha256-rlG+UKI9O4fu39GcCK03pRSbx8YmSjGVlZG6WIUcbuU=";
              "wonopcode-storage-0.1.2" = "sha256-rlG+UKI9O4fu39GcCK03pRSbx8YmSjGVlZG6WIUcbuU=";
              "wonopcode-util-0.1.2" = "sha256-rlG+UKI9O4fu39GcCK03pRSbx8YmSjGVlZG6WIUcbuU=";
              "wonopcode-snapshot-0.1.2" = "sha256-rlG+UKI9O4fu39GcCK03pRSbx8YmSjGVlZG6WIUcbuU=";
              "wonopcode-lsp-0.1.2" = "sha256-rlG+UKI9O4fu39GcCK03pRSbx8YmSjGVlZG6WIUcbuU=";
              "wonopcode-mcp-0.1.2" = "sha256-rlG+UKI9O4fu39GcCK03pRSbx8YmSjGVlZG6WIUcbuU=";
              "wonopcode-sandbox-0.1.2" = "sha256-rlG+UKI9O4fu39GcCK03pRSbx8YmSjGVlZG6WIUcbuU=";
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
                                export LD_LIBRARY_PATH="${pkgs.lib.makeLibraryPath [ pkgs.zlib pkgs.zstd pkgs.openssl ]}:$LD_LIBRARY_PATH"
                              ''
                    
           + pkgs.lib.optionalString isDarwin ''
            # Nix can set TMPDIR to a shell-specific path on macOS; use a stable path.
            export TMPDIR=/var/tmp
          '' + pkgs.lib.optionalString isLinux ''
            # Expose the CUDA toolkit for Candle CUDA builds.
            export CUDA_PATH="${pkgs.cudatoolkit}"
            export CUDA_ROOT="${pkgs.cudatoolkit}"
            export CUDA_TOOLKIT_ROOT_DIR="${pkgs.cudatoolkit}"
            # Prefer the real host CUDA driver at runtime without relying on
            # LD_LIBRARY_PATH, while still using the toolkit stubs for linking.
            cuda_driver_rpath=""
            for candidate in \
              /run/opengl-driver/lib \
              /usr/lib/x86_64-linux-gnu \
              /usr/lib/wsl/lib
            do
              if [ -f "$candidate/libcuda.so.1" ]; then
                cuda_driver_rpath="$candidate"
                break
              fi
            done

            linux_link_args="-C link-arg=-fuse-ld=mold"
            if [ -n "$cuda_driver_rpath" ]; then
              linux_link_args="$linux_link_args -C link-arg=-Wl,-rpath,$cuda_driver_rpath"
            fi

            export CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_RUSTFLAGS="''${CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_RUSTFLAGS:+$CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_RUSTFLAGS }$linux_link_args"
            export CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_RUSTFLAGS="''${CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_RUSTFLAGS:+$CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_RUSTFLAGS }-C link-arg=-fuse-ld=mold"
          '';
        };
      });
}
