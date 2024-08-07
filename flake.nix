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
          version = "0.4.0";
          src = ./.;
          cargoHash = "sha256-h0hP+N+vuSKr58IHD9x7q+ZL3qyPI1f3Sl/IAfJA5JY=";
          buildInputs = with pkgs; [
            cargo
            openssl
            pkg-config
            rustc
            rustfmt
          ];
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
