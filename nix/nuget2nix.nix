{inputs, ...}: {
  perSystem = {
    pkgs,
    system,
    config,
    ...
  }: let
    inherit (pkgs) lib stdenv;
    toolchain = inputs.fenix.packages.${system}.minimal.toolchain;
  in {
    packages = {
      nuget2nix =
        (pkgs.makeRustPlatform {
          cargo = toolchain;
          rustc = toolchain;
        })
        .buildRustPackage {
          pname = "nuget2nix";
          version = (lib.importTOML ./../Cargo.toml).package.version;
          src = ./..;
          cargoLock.lockFile = ./../Cargo.lock;
          nativeBuildInputs = [ pkgs.pkg-config ];
          buildInputs = [ pkgs.pkg-config pkgs.openssl ];
        };
    };
  };
}
