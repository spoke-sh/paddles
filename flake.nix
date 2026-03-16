{
  description = "paddles development environment";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    rust-overlay.url = "github:oxalica/rust-overlay";
    flake-utils.url = "github:numtide/flake-utils";

    sift.url = "github:rupurt/sift?ref=main";

    keel = {
      url = "git+ssh://git@github.com/spoke-sh/keel.git?rev=0e078226c7bf61de9c1c6c6261697bc332411cf2";
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
      in {
        packages = {
          keel = keelPkg;
          sift = siftPkg;
        };
        # Expose keelPkg as the default package for the system
        defaultPackage = keelPkg; 

        devShells.default = pkgs.mkShell {
                    buildInputs = [
                      rust
                      pkgs.just
                      pkgs.cargo-nextest
                      pkgs.cargo-llvm-cov
                      pkgs.pkg-config
                      pkgs.openssl
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
          '' + pkgs.lib.optionalString isDarwin ''
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
