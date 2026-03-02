{
  description = "An EverQuest .wld file loader";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";
    rust-overlay.url = "github:oxalica/rust-overlay";
    crane.url = "github:ipetkov/crane";
  };

  outputs = { self, nixpkgs, rust-overlay, crane }:
    let
      systems = [ "x86_64-linux" "aarch64-linux" "x86_64-darwin" "aarch64-darwin" ];
      forAllSystems = nixpkgs.lib.genAttrs systems;
      forSystem = system:
        let
          pkgs = import nixpkgs {
            inherit system;
            overlays = [ rust-overlay.overlays.default ];
          };
          rust = pkgs.rust-bin.stable.latest.default;
          rustDev = rust.override {
            extensions = [ "rust-src" "rust-analyzer" ];
          };
          craneLib = (crane.mkLib pkgs).overrideToolchain rust;

          src = pkgs.lib.cleanSourceWith {
            src = ./.;
            filter = path: type:
              (craneLib.filterCargoSources path type)
              || builtins.match ".*README\\.md$" path != null;
          };

          commonArgs = {
            inherit src;
            strictDeps = true;
          };

          cargoArtifacts = craneLib.buildDepsOnly commonArgs;
        in
        {
          inherit pkgs rustDev craneLib commonArgs cargoArtifacts;
        };
    in
    {
      overlays.default = final: prev: {
        s3d = self.packages.${final.system}.s3d;
        wld-cli = self.packages.${final.system}.wld-cli;
      };

      packages = forAllSystems (system:
        let
          s = forSystem system;
        in
        {
          s3d = s.craneLib.buildPackage (s.commonArgs // {
            inherit (s) cargoArtifacts;
            pname = "s3d";
            cargoExtraArgs = "--package s3d";
          });

          wld-cli = s.craneLib.buildPackage (s.commonArgs // {
            inherit (s) cargoArtifacts;
            pname = "wld-cli";
            cargoExtraArgs = "--package wld-cli";
          });
        });

      devShells = forAllSystems (system:
        let
          s = forSystem system;
        in
        {
          default = s.pkgs.mkShell {
            buildInputs = [
              s.rustDev
              s.pkgs.pkg-config
              s.pkgs.openssl
            ];
            nativeBuildInputs = [
              s.pkgs.nixpkgs-fmt
              s.pkgs.perf-tools
              s.pkgs.cargo-flamegraph
              s.pkgs.cargo-readme
            ];
          };
        });
    };
}
