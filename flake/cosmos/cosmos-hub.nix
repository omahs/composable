{ self, ... }: {
  perSystem = { self', pkgs, systemCommonRust, subnix, lib, system, devnetTools
    , cosmosTools, bashTools, ... }:
    let devnet-root-directory = cosmosTools.devnet-root-directory;
    in {

      packages = rec {
        gaiad = pkgs.writeShellApplication {
          name = "gaiad";
          runtimeInputs = devnetTools.withBaseContainerTools;
          text = ''
            ${self.inputs.cosmos.packages.${system}.gaia}/bin/gaiad "$@"
          '';
        };
      };
    };
}