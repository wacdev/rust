#!/usr/bin/env bash

DIR=$(realpath $0) && DIR=${DIR%/*}
cd $DIR
set -ex

IMG=art/img

NAME=$(basename $IMG)

(docker stop $NAME || true) && exec docker run --rm --name $NAME $IMG
