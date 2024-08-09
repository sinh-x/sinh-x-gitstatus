{
  description = "Sinh-x-gitstatus";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs?ref=nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
  };
  outputs = {
    self,
    nixpkgs,
    flake-utils,
    ...
  }:
    flake-utils.lib.eachDefaultSystem (
      system: let
        pkgs = nixpkgs.legacyPackages.${system};
        sinh-x-gitstatus = pkgs.rustPlatform.buildRustPackage {
          pname = "sinh-x-gitstatus";
          version = "0.5.1";
          src = ./.;
          cargoHash = "sha256-9/BKrKaL+W9Qjlm/qXED//6kzMNTP2dyHXiiXPUi3QQ=";
          buildInputs = with pkgs; [
            cargo
            openssl
            pkg-config
            rustc
            rustfmt
          ];
          nativeBuildInputs = with pkgs; [
            pkg-config
            openssl
          ];
          cargoBuildFlags = ["--release"];
        };
      in {
        defaultPackage = sinh-x-gitstatus;

        devShell = pkgs.mkShell {
          buildInputs = with pkgs; [
            cargo
            openssl
            pkg-config
            rustc
            rustfmt
          ];
          shellHook = ''
            exec fish
          '';
        };
      }
    );
}
