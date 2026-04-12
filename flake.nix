{
  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable-small";
    systems.url = "github:nix-systems/default";
    flakeCompat = {
      url = "github:edolstra/flake-compat";
      flake = false;
    };
  };

  outputs = {self, nixpkgs, systems, flakeCompat}:
    let
      forEachSystem = nixpkgs.lib.genAttrs (import systems);
      pkgs = forEachSystem (system: nixpkgs.legacyPackages.${system});
    in
    {
      packages = forEachSystem (system: {
        default = pkgs.${system}.callPackage ./pkgs/default {};
        ci = pkgs.${system}.callPackage ./pkgs/ci {};
      });

      checks = forEachSystem (system: {
        default = self.packages.${system}.default;
      });

      devShells = forEachSystem (system: {
        default = pkgs.${system}.mkShell {
          packages = [
            pkgs.${system}.git
            pkgs.${system}.cargo
            pkgs.${system}.rustc
            pkgs.${system}.rustfmt
            pkgs.${system}.clippy
            pkgs.${system}.cargo-tarpaulin
            self.packages.${system}.ci
          ];
        };
      });
    };
}
