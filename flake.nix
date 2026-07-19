{
  description = "The Ultimate Systems Programming Academy — a dev shell with Rust, Go, Python, uv, and Node, so every example and the site build reproducibly.";

  inputs.nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
  inputs.flake-utils.url = "github:numtide/flake-utils";

  outputs = { self, nixpkgs, flake-utils }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs { inherit system; };
      in
      {
        devShells.default = pkgs.mkShell {
          # Everything the examples and the docs site need — pinned by the flake
          # lock, so `nix develop` gives the same toolchain on every machine.
          packages = with pkgs; [
            # Rust: cargo, rustc, clippy, rustfmt
            rustc
            cargo
            clippy
            rustfmt
            # Go
            go
            # Python + uv + ruff
            python312
            uv
            ruff
            # Node, for the Astro docs site
            nodejs_22
          ];

          shellHook = ''
            echo "Systems Programming Academy dev shell"
            echo "  rustc $(rustc --version | cut -d' ' -f2) · go $(go version | cut -d' ' -f3) · python $(python3 --version | cut -d' ' -f2)"
            echo "  run every example suite:  scripts/run-all.sh"
            echo "  build the docs site:      npm install && npm run build"
          '';
        };
      });
}
