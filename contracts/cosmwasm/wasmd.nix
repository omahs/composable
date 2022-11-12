{ self, ... }: {
  perSystem = { config, self', inputs', pkgs, system, ... }: {
    packages = {
      wasmd = pkgs.stdenv.mkDerivation rec {
        name = "wasmd";
        src = pkgs.fetchFromGitHub {
          repo = "wasmd";
          owner = "CosmWasm";
          rev = "ef9a84dda82538265ce1686812481ebc58da097c";
          sha256 =
            "sha256:2ca8z33pnn6x9dkxii70s1lcskh56fzng1x9lqxzk84q5fffysdb";
        };
      };
    };
  };
}
