{
  description = "Open Modular";

  inputs = {
    flake-utils.url = "github:numtide/flake-utils";
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";

    rust-overlay = {
      url = "github:oxalica/rust-overlay";

      inputs = {
        nixpkgs.follows = "nixpkgs";
      };
    };
  };

  outputs = {
    flake-utils,
    nixpkgs,
    rust-overlay,
    ...
  }:
    flake-utils.lib.eachDefaultSystem (
      system: let
        overlays = [(import rust-overlay)];
        pkgs = import nixpkgs {
          inherit system overlays;
        };
        rust = (
          if builtins.pathExists ./rust-toolchain.toml
          then pkgs.rust-bin.fromRustupToolchainFile ./rust-toolchain.toml
          else if builtins.pathExists ./rust-toolchain
          then pkgs.rust-bin.fromRustupToolchainFile ./rust-toolchain
          else
            pkgs.rust-bin.stable.latest.default.override {
              extensions = [
                "rust-src"
                "rustfmt"
              ];
            }
        );
      in
        with pkgs; {
          devShells.default = mkShell {
            buildInputs = [
              cmake
              gnuplot
              rust
            ];

            shellHook = ''
              export CARGO_HOME=$(realpath ./.cargo)
              export PATH=$(realpath ./.cargo/bin):$PATH
            '';
          };
        }
    );
}
