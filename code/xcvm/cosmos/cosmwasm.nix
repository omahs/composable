{ self, ... }: {
  perSystem = { config, self', inputs', pkgs, system, ... }: {
    packages = {
      wasmd = pkgs.stdenv.mkDerivation rec {
        name = "wasmd";
        src = pkgs.fetchFromGitHub {
          repo = "wasmd";
          owner = "CosmWasm";
          rev = "ef9a84dda82538265ce1686812481ebc58da097c";
          sha256 = "sha256-roQ6fAHT1pdzeaLjedStg+C8voDnj8gbo/R0zloXZlo=";
        };
      };
      cosmwasm-check = rustPlatform.buildRustPackage rec {
        pname = "cosmwasm-check";
        version = "0948715950151579aaba487944b630332d83e215";

        src = fetchFromGitHub {
          owner = "CosmWasm";
          repo = "cosmwasm";
          rev = version;
          sha256 = "1BuviHXWfvnZh/68qyDNlJatb/aHjmOszL1uS5W4IMQ=";
        };
        # nativeBuildInputs = [ pkg-config ];
        # buildInputs = [ openssl ];

        cargoSha256 = "UYTlhV15GYFOFPZ15fsHlWBwp/guCe8yh7JiJB/AVwE=";
      };
    };
  };
}
