{inputs, ...}: {
  perSystem = {system, ...}: let
    fenix = inputs.fenix.packages.${system};
  in {
    devshells.default = {
      packages = [
        (fenix.complete.withComponents [
          "cargo"
          "clippy"
          "rust-src"
          "rustc"
          "rustfmt"
        ])
        fenix.rust-analyzer
      ];
    };
  };
}
