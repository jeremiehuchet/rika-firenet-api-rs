#!/bin/env sh
set -e

workdir=$(cd $(dirname $0) ; pwd)

openapi-generator-cli generate \
        -g rust \
        -o $workdir \
        -i $workdir/rika-firenet-openapi.yml \
        -c $workdir/openapi-config.yml