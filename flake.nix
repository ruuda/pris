{
  description = "Pris";

  inputs.nixpkgs.url = "nixpkgs/nixos-23.05";

  outputs = { self, nixpkgs }:
    let
      supportedSystems = ["x86_64-linux" "aarch64-linux" "aarch64-darwin"];
      # Ridiculous boilerplate required to make flakes somewhat usable.
      forEachSystem = f:
        builtins.zipAttrsWith
          (name: values: builtins.foldl' (x: y: x // y) {} values)
          (map
            (k: builtins.mapAttrs (name: value: { "${k}" = value; }) (f k))
            supportedSystems
          );
    in
      forEachSystem (system:
        let
          name = "pris";
          version = builtins.substring 0 8 self.lastModifiedDate;
          pkgs = import nixpkgs { inherit system; };
          python = pkgs.python3.withPackages (ps: [
            ps.scipy
            ps.numpy
            ps.matplotlib
          ]);
          buildInputs = [
            pkgs.cairo
            pkgs.fontconfig
            pkgs.freetype
            pkgs.harfbuzz
            pkgs.librsvg
          ];
        in
          rec {
            devShells.default = pkgs.mkShell {
              inherit name;
              nativeBuildInputs = [
                pkgs.mkdocs
                pkgs.pkg-config
                pkgs.python3
                pkgs.rustup
              ] ++ buildInputs;
            };

            packages.default = pkgs.rustPlatform.buildRustPackage {
              inherit name version buildInputs;
              src = ./.;
              cargoLock = {
                lockFile = ./Cargo.lock;
              };
              nativeBuildInputs = [ pkgs.pkg-config ];
            };
          }
      );
}
