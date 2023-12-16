{ pkgs, ... }:

{
  packages = [
    pkgs.openapi-generator-cli
    pkgs.openssl
    pkgs.pkg-config
    pkgs.vscode
  ];

  languages.rust = {
    enable = true;
    channel = "stable";
  };

  pre-commit.hooks.rustfmt.enable = true;

  scripts.docker-build-mock.exec = ''
    docker build \
        --file $DEVENV_ROOT/mock/Dockerfile \
        --tag rika-firenet-api-mock:latest \
        $DEVENV_ROOT/mock
  '';
  scripts.openapi-generate.exec = ''
    openapi-generator-cli generate \
        -g rust \
        -o $DEVENV_ROOT/rika-firenet-openapi \
        -i $DEVENV_ROOT/rika-firenet-openapi/rika-firenet-openapi.yml \
        -c $DEVENV_ROOT/rika-firenet-openapi/openapi-config.yml
  '';
}
