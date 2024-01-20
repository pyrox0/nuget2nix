{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    flake-parts.url = "github:hercules-ci/flake-parts";
    devshell.url = "github:numtide/devshell";
    devshell.inputs.nixpkgs.follows = "nixpkgs";
    fenix.url = "github:nix-community/fenix";
    fenix.inputs.nixpkgs.follows = "nixpkgs";
  };

  outputs = inputs @ {
    flake-parts,
    fenix,
    ...
  }:
    flake-parts.lib.mkFlake {inherit inputs;} {
      imports = [
        inputs.devshell.flakeModule
        {
          perSystem = {
            pkgs,
            system,
            config,
            ...
          }: let
            inherit (pkgs) lib stdenv;
            toolchain = pkgs.fenix.minimal.toolchain;
          in {
            _module.args.pkgs = import inputs.nixpkgs {
              inherit system;
              overlays = [fenix.overlays.default];
            };

            packages = rec {
              nuget2nix =
                (pkgs.makeRustPlatform {
                  cargo = toolchain;
                  rustc = toolchain;
                })
                .buildRustPackage {
                  pname = "nuget2nix";
                  version = (lib.importTOML ./Cargo.toml).package.version;
                  src = ./.;
                  cargoLock.lockFile = ./Cargo.lock;

                  buildInputs = lib.optional stdenv.isDarwin [
                    pkgs.darwin.apple_sdk.frameworks.Security
                    pkgs.darwin.apple_sdk.frameworks.SystemConfiguration
                  ];
                };

              default = nuget2nix;
            };

            apps = rec {
              nuget2nix = {
                type = "app";
                program = "${config.packages.nuget2nix}/bin/nuget2nix";
              };
              default = nuget2nix;
            };

            devshells.default = {
              packages = [
                (pkgs.fenix.complete.withComponents [
                  "cargo"
                  "clippy"
                  "rust-src"
                  "rustc"
                  "rustfmt"
                ])
                pkgs.rust-analyzer-nightly
              ];
            };
          };
        }
      ];
      systems = ["aarch64-darwin" "x86_64-darwin" "x86_64-linux"];
    };
}
