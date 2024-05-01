{
  inputs = {
    flake-utils.url = "github:numtide/flake-utils";
    # rainix.url = "github:rainprotocol/rainix";
    rust-overlay.url = "github:oxalica/rust-overlay";
  };

  outputs = { self, flake-utils, rust-overlay }:

  flake-utils.lib.eachDefaultSystem (system:
    let
      overlays =[ (import rust-overlay) ];
        pkgs = import nixpkgs {
          inherit system overlays;
        };
        rust-version = "1.76.0";
        rust-toolchain = pkgs.rust-bin.stable.${rust-version}.default.override (previous: {
          targets = previous.targets ++ [ "wasm32-unknown-unknown" ];
        });
    in {
      # For `nix develop`:
      devShell = pkgs.mkShell {
        nativeBuildInputs = [
          rust-toolchain
          pkgs.cargo-release
          pkgs.gmp
          pkgs.openssl
          pkgs.libusb
          pkgs.pkg-config
          pkgs.wasm-bindgen-cli
          pkgs.gettext
          pkgs.libiconv
          pkgs.cargo-flamegraph
          # rainix.rust-build-inputs.${system}
          # rainix.packages.${system}.rainix-rs-test
          # rainix.packages.${system}.rainix-rs-artifacts
          # rainix.packages.${system}.rainix-rs-prelude
          # rainix.packages.${system}.rainix-rs-static
        ];
      };
    }
  );
}