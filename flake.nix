{
  description = "Development Nix flake for OpenAI Codex CLI";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = { nixpkgs, flake-utils, rust-overlay, ... }: 
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs {
          inherit system;
        };
        pkgsWithRust = import nixpkgs {
          inherit system;
          overlays = [ rust-overlay.overlays.default ];
        };
        monorepo-deps = with pkgs; [
          # for precommit hook
          pnpm
          husky
        ];
        kay-rs = import ./kay-rs {
          pkgs = pkgsWithRust;
          inherit monorepo-deps;
        };
      in
      {
        packages = {
          kay-rs = kay-rs.package;
          code-rs = kay-rs.package;
          default = kay-rs.package;
        };

        devShells = {
          kay-rs = kay-rs.devShell;
          code-rs = kay-rs.devShell;
          default = kay-rs.devShell;
        };

        apps = {
          kay-rs = kay-rs.app;
          code-rs = kay-rs.app;
          default = kay-rs.app;
        };
      }
    );
}
