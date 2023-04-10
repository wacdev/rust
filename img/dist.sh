#!/usr/bin/env bash

DIR=$(realpath $0) && DIR=${DIR%/*}
cd $DIR
set -ex

if ! [ -x "$(command -v cargo)" ]; then
  cargo_env="$HOME/.cargo/env"
  if [ -f "$cargo_env" ]; then
    source $cargo_env
  fi
fi

./sh/jpegxl-rs.sh

target=$(rustc -vV | grep "host:" | awk '{print $2}')

export RUSTFLAGS='--cfg reqwest_unstable'

cargo build --release --target $target

