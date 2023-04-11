#!/usr/bin/env bash

DIR=$(realpath $0) && DIR=${DIR%/*}
cd $DIR
set -ex

cargo build
RUST_BACKTRACE=full exec ./target/debug/img
