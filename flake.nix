{
  description = "command-line-rust";

  inputs = {
    fenix = {
      url = "github:nix-community/fenix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    flake-parts.url = "github:hercules-ci/flake-parts";
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
  };

  outputs =
    inputs@{ fenix, flake-parts, ... }:
    flake-parts.lib.mkFlake { inherit inputs; } {
      systems = [
        "aarch64-darwin"
        "aarch64-linux"
        "x86_64-darwin"
        "x86_64-linux"
      ];
      perSystem =
        { pkgs, system, ... }:
        {
          _module.args.pkgs = import inputs.nixpkgs {
            inherit system;
            overlays = [
              fenix.overlays.default
            ];
          };
          apps = {
            commitlint = {
              type = "app";
              program = "${pkgs.commitlint}/bin/commitlint";
            };
            oxfmt = {
              type = "app";
              program = "${pkgs.oxfmt}/bin/oxfmt";
            };
          };
          devShells.default = pkgs.mkShell {
            packages = with pkgs; [
              (pkgs.fenix.fromToolchainFile {
                file = ./rust-toolchain.toml;
                sha256 = "qqF33vNuAdU5vua96VKVIwuc43j4EFeEXbjQ6+l4mO4=";
              })
              commitlint
              editorconfig-checker
              lefthook
              oxfmt
              rust-analyzer
              yamllint
            ];
          };
          formatter = pkgs.nixfmt-tree;
        };
    };
}
