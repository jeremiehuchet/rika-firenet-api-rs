let
  # release 22.11, 2023-02-20
  pkgs = import (fetchTarball("https://github.com/NixOS/nixpkgs/archive/cd56369075acb5fa8459e640b56a0dc1f05e1022.tar.gz")) {};
  openapi-generator-cli-6_4 = pkgs.openapi-generator-cli.overrideAttrs (finalAttrs: previousAttrs: {
    version = "6.4.0";
    src = pkgs.fetchurl {
      url = "mirror://maven/org/openapitools/${previousAttrs.pname}/${finalAttrs.version}/${previousAttrs.pname}-${finalAttrs.version}.jar";
      sha256 = "sha256-Na6tMA4MlGn72dMM9G9BU4l9yygpEgkcpOySEtzp0VE=";
    };
  });
in pkgs.mkShell {
  buildInputs = with pkgs; [
    cargo
    openapi-generator-cli-6_4
    openssl
    pkg-config
    rustc
    rustfmt
    vscode
  ];
  RUST_SRC_PATH = "${pkgs.rust.packages.stable.rustPlatform.rustLibSrc}";
}
