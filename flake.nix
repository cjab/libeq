{
  description = "An EverQuest .wld file loader";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";
    rust-overlay.url = "github:oxalica/rust-overlay";
  };

  outputs = { self, nixpkgs, rust-overlay }:
    let
      systems = [ "x86_64-linux" "aarch64-linux" "x86_64-darwin" "aarch64-darwin" ];
      forAllSystems = nixpkgs.lib.genAttrs systems;
    in
    {
      devShells = forAllSystems
        (system:
          let
            pkgs = import nixpkgs {
              inherit system;
              overlays = [ rust-overlay.overlays.default ];
            };
            rust = pkgs.rust-bin.stable.latest.default.override {
              extensions = [ "rust-src" "rust-analyzer" ];
            };
          in
          {
            default = pkgs.mkShell {
              buildInputs = [
                rust
                pkgs.pkg-config
                pkgs.openssl
              ];
              nativeBuildInputs = [
                pkgs.nixpkgs-fmt
                pkgs.perf-tools
                pkgs.cargo-flamegraph
                pkgs.cargo-readme
              ];
            };
          });
    };
}
