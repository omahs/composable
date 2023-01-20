{ self, ... }: {
  perSystem = { config, self', inputs', pkgs, system, crane, ... }: {
    packages = rec {
      hyperspace-dali-container = pkgs.dockerTools.buildImage {
        tag = "latest";
        name = "hyperspace-dali";
        config = { Entrypoint = [ "${hyperspace-dali}/bin/hyperspace" ]; };
      };

      hyperspace-dali = let
        src = pkgs.stdenv.mkDerivation rec {
          name = "centauri";
          pname = "${name}";
          buildInputs = [ self'.packages.dali-subxt-client ];
          src = pkgs.fetchFromGitHub {
            owner = "obsessed-cake";
            repo = "centauri";
            rev = "050b4eb23aa221e7ebec1bd4c1663f784744e50f";
            hash = "sha256-klGg48in0VhqG/V4ZQ0t1v2vE429IlOXolREiCKczK8=";
          };
          installPhase = ''
            mkdir $out
            cp -a $src/. $out/
            chmod u+w $out/utils/subxt/generated/src/{parachain.rs,relaychain.rs}
            cp ${self'.packages.dali-subxt-client}/* $out/utils/subxt/generated/src/
          '';
        };
      in crane.stable.buildPackage {
        name = "hyperspace-dali";
        cargoArtifacts = crane.stable.buildDepsOnly {
          inherit src;
          doCheck = false;
          cargoExtraArgs = "-p hyperspace --features dali";
          cargoTestCommand = "";
          BuildInputs = [ pkgs.protobuf ];
          PROTOC = "${pkgs.protobuf}/bin/protoc";
          PROTOC_INCLUDE = "${pkgs.protobuf}/include";
          PROTOC_NO_VENDOR = "1";
        };
        inherit src;
        BuildInputs = [ pkgs.protobuf ];
        PROTOC = "${pkgs.protobuf}/bin/protoc";
        PROTOC_INCLUDE = "${pkgs.protobuf}/include";
        PROTOC_NO_VENDOR = "1";
        doCheck = false;
        cargoExtraArgs = "-p hyperspace --features dali";
        cargoTestCommand = "";
        meta = { mainProgram = "hyperspace"; };
      };
    };
  };
}
