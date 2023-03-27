#!/bin/env sh
set -e

openapi-generator-cli generate \
        -g rust \
        -o rika-firenet-openapi \
        -i rika-firenet-openapi/rika-firenet-openapi.yml \
        -c rika-firenet-openapi/openapi-config.yml