let
  # release 23.05, 2023-08-13
  pkgs = import (fetchTarball("https://github.com/NixOS/nixpkgs/archive/90497216e09e9fe341fc3f2544398000cad33d20.tar.gz")) {};
  openapi-generator-cli-6_4 = pkgs.openapi-generator-cli.overrideAttrs (finalAttrs: previousAttrs: {
    version = "6.4.0";
    src = pkgs.fetchurl {
      url = "mirror://maven/org/openapitools/${previousAttrs.pname}/${finalAttrs.version}/${previousAttrs.pname}-${finalAttrs.version}.jar";
      sha256 = "sha256-Na6tMA4MlGn72dMM9G9BU4l9yygpEgkcpOySEtzp0VE=";
    };
  });
in pkgs.mkShell {
  buildInputs = with pkgs; [
    openapi-generator-cli-6_4
    openssl
    pkg-config
    rust.packages.stable.cargo
    rust.packages.stable.rustc
    rust.packages.stable.rustfmt
    vscode
  ];
  RUST_SRC_PATH = "${pkgs.rust.packages.stable.rustPlatform.rustLibSrc}";
}
