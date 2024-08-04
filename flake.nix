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
          version = "0.1.0";
          src = ./.;
          cargoHash = "sha256-P35+K7ipaPg7z1HXPjofEW4LM21VDsCyJP/SidMnrik=";
          buildInputs = with pkgs; [
            cargo
            llvmPackages.clang
            llvmPackages.libclang
            openssl
            pkg-config
            rustc
            rustfmt
          ];
          nativeBuildInputs = with pkgs; [
            cargo
            openssl
            pkg-config
            rustc
          ];
        };
      in {
        defaultPackage = sinh-x-gitstatus;

        devShell = pkgs.mkShell {
          buildInputs = with pkgs; [
            cargo
            llvmPackages.clang
            llvmPackages.libclang
            openssl
            pkg-config
            rustc
            rustfmt
          ];
          LIBCLANG_PATH = "${pkgs.llvmPackages.libclang.lib}/lib";
          shellHook = ''
            exec fish
          '';
        };
      }
    );
}
