{
  lib,
  flake-parts-lib,
  ...
}: let
  inherit (lib) mkOption types;
  inherit (flake-parts-lib) mkTransposedPerSystemModule;
in {
  imports = [
    (mkTransposedPerSystemModule {
      name = "buildDotnetModule";
      option = mkOption {
        type = types.functionTo types.package;
        description = "Utility nix functions";
      };
      file = ./lib.nix;
    })
  ];

  perSystem = {
    pkgs,
    system,
    config,
    ...
  }: {
    buildDotnetModule = pkgs.buildDotnetModule.override {
      nuget-to-nix = pkgs.callPackage ({...}:
        pkgs.writeShellApplication {
          name = "nuget-to-nix";
          runtimeInputs = [config.packages.nuget2nix];
          text = ''
            nuget2nix "$1" "$2"
          '';
        }) {};
    };
  };
}
