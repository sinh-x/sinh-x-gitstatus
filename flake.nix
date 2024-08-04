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
          cargoHash = "sha256-9FxiECz5na/Ah3RvI+v7xVl7L+JSgDiiX92KKxhcXfM=";
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
            llvmPackages.clang
            llvmPackages.libclang
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
