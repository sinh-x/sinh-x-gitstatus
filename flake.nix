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
          version = "0.5.0";
          src = ./.;
          cargoHash = "sha256-1iu4GPpCMRfZfY1UlQ7669bLSdHGKg8k9zKESyRbfVI=";
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
          buildPhase = ''
            echo $PATH
            # Call the default buildPhase
            runHook preBuild
            cargo build --release
            runHook postBuild
          '';
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
