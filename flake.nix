{
  description = "Rust development environment with fenix";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-26.05";
    fenix = {
      url = "github:nix-community/fenix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, fenix, flake-utils }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = nixpkgs.legacyPackages.${system};
        toolchain = fenix.packages.${system}.complete.toolchain;
      in
      {
        devShells.default = pkgs.mkShell {
          buildInputs = with pkgs; [
            toolchain # Rustc, Cargo, Clippy, Rustfmt, Rust-src
            rust-analyzer # LSP
            cargo-watch
            cargo-edit
            cargo-expand
          ];

          # Environment variables for Rust development
          RUST_SRC_PATH = "${toolchain}/lib/rustlib/src/rust/library";
          CARGO_TARGET_DIR = "target";

          shellHook = ''
            echo "🔧 Rust $(rustc --version) ready"
            echo "📦 Cargo $(cargo --version)"
            echo "✨ Using fenix toolchain"
          '';
        };
      });
}
