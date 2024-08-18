{
  # Snowfall Lib provides a customized `lib` instance with access to your flake's library
  # as well as the libraries available from your flake's inputs.
  # You also have access to your flake's inputs.

  # The namespace used for your flake, defaulting to "internal" if not set.

  # All other arguments come from NixPkgs. You can use `pkgs` to pull packages or helpers
  # programmatically or you may add the named attributes as arguments here.
  pkgs,
  ...
}:
pkgs.rustPlatform.buildRustPackage {
  pname = "sinh-x-gitstatus";
  version = "0.6.0";
  src = ../..;
  cargoHash = "sha256-lMJZkZRc7k57Aic17If6otscLmk8Io4020/tt1w5Pw0=";
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
  cargoBuildFlags = [ "--release" ];
}
