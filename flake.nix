{
  inputs = {
    flake-utils.url = "github:numtide/flake-utils";
    rainix.url = "github:rainlanguage/rainix";
  };

  outputs =
    {
      flake-utils,
      rainix,
    }:

    flake-utils.lib.eachDefaultSystem (system: {
      devShells.default = rainix.devShells.${system}.default;
    });
}
