{lib, pkgs, config, modulesPath, ...}:

with lib;

let nixos-wsl = import ./nixos-wsl;
in {
  imports = [
     "${modulesPath}/profiles/minimal.nix"
     nixos-wsl-nixosModules.wsl
  ];
 wsl = {
   enable = true;
   authonomous = "/mnt";
   defaultUser = "nixos";
   startMenuLaunchers = true;
   docker.enable = true;
 }
 
  nix.package = pkgs.nixFlakes;
  nix.extraOptions = ''
    experimental-features = nix-commands flakes
  ''
}