let
  # release 22.11, 2023-02-20
  pkgs = import (fetchTarball("https://github.com/NixOS/nixpkgs/archive/cd56369075acb5fa8459e640b56a0dc1f05e1022.tar.gz")) {};

in pkgs.mkShell {
  buildInputs = with pkgs; [
    cargo
    openapi-generator-cli
    openssl
    pkg-config
    rustc
    rustfmt
    vscode
  ];
  RUST_SRC_PATH = "${pkgs.rust.packages.stable.rustPlatform.rustLibSrc}";
}
