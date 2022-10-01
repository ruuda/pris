let
  pkgs = import ./nixpkgs-pinned.nix {};
in
  pkgs.buildEnv {
    name = "pris-devenv";
    paths = [
      pkgs.mkdocs  # For building documentation.
      pkgs.python3 # For running the tests.
      pkgs.rustup  # Provides the Rust toolchain.
    ];
  }
