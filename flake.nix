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
          version = "0.2.0";
          src = ./.;
          cargoHash = "sha256-8K0t3/lVVfkF/cl0E6OCP1z0Qpe05Rzn01NX51HyzHo=";
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
