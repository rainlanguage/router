{
  inputs = {
    flake-utils.url = "github:numtide/flake-utils";
    rainix.url = "github:rainprotocol/rainix";
  };

  outputs = { self, flake-utils, rainix }:

  flake-utils.lib.eachDefaultSystem (system:
    let
      pkgs = rainix.pkgs.${system};
    in {
      # For `nix develop`:
      devShell = pkgs.mkShell {
        buildInputs = [
          pkgs.libressl
          rainix.rust-build-inputs.${system}
          rainix.packages.${system}.rainix-rs-test
          rainix.packages.${system}.rainix-rs-artifacts
          rainix.packages.${system}.rainix-rs-prelude
          rainix.packages.${system}.rainix-rs-static
        ];
      };
    }
  );
}