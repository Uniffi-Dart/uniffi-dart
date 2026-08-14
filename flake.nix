{
  description = "uniffi-dart";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-25.11";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    treefmt-nix = {
      url = "github:numtide/treefmt-nix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs =
    {
      self,
      nixpkgs,
      flake-utils,
      rust-overlay,
      treefmt-nix,
    }:
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        msrvVersion = "1.91.0";

        rustExtensions = [
          "clippy"
          "rust-src"
          "rustfmt"
        ];

        pkgs = import nixpkgs {
          inherit system;
          overlays = [
            rust-overlay.overlays.default
            (_final: prev: {
              rustToolchains = {
                msrv = prev.rust-bin.stable.${msrvVersion}.default.override {
                  extensions = rustExtensions;
                };
                stable = prev.rust-bin.stable.latest.default.override {
                  extensions = rustExtensions;
                };
                nightly = prev.rust-bin.nightly.latest.default.override {
                  extensions = rustExtensions;
                };
              };
            })
          ];
        };

        rustToolchains = pkgs.rustToolchains;

        treefmtEval = treefmt-nix.lib.evalModule pkgs ./treefmt.nix;

        devPackages = with pkgs; [
          cargo-edit
          cargo-nextest
          cargo-watch
          codespell
          dart
          git
          pkg-config
          rust-analyzer
          taplo
        ];

        shellHook = ''
          export CARGO_WORKSPACE_ROOT="$(git rev-parse --show-toplevel 2>/dev/null || pwd)"
        '';

        devShells = builtins.mapAttrs (
          _name: rustToolchain:
          pkgs.mkShell {
            packages = [
              rustToolchain
            ]
            ++ devPackages;
            inherit shellHook;
          }
        ) rustToolchains;
      in
      {
        devShells = devShells // {
          default = devShells.stable;
        };

        formatter = treefmtEval.config.build.wrapper;

        checks = {
          formatting = treefmtEval.config.build.check self;
        };
      }
    );
}
