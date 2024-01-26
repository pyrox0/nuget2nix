{
  description = "treefmt nix configuration modules";
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    flake-parts.url = "github:hercules-ci/flake-parts";
    devshell.url = "github:numtide/devshell";
    devshell.inputs.nixpkgs.follows = "nixpkgs";
    fenix.url = "github:nix-community/fenix";
    fenix.inputs.nixpkgs.follows = "nixpkgs";
  };

  outputs = inputs @ {flake-parts, ...}:
    flake-parts.lib.mkFlake {inherit inputs;} (let
      devshell = ./nix/devshell.nix;
      flakeModule = ./flake-module.nix;
    in {
      imports = [
        inputs.devshell.flakeModule
        devshell
        flakeModule
      ];

      flake = {
        inherit flakeModule;
      };

      perSystem = {config, ...}: {
        packages.default = config.packages.nuget2nix;
      };

      systems = ["aarch64-darwin" "x86_64-darwin" "x86_64-linux"];
    });
}
