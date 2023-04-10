#!/usr/bin/env bash

DIR=$(realpath $0) && DIR=${DIR%/*/*}
set -ex
cd $DIR

os_type=$(uname)

if [ ! -d "jpegxl-rs" ]; then
  git clone --recursive --shallow-submodules --depth=1 \
    https://github.com/inflation/jpegxl-rs.git

  # if [ "$os_type" != "Linux" ]; then
  cd jpegxl-rs/jpegxl-src/libjxl
  ./deps.sh && cmake . && make && make install
  rm CMakeCache.txt
  # fi

  cd $DIR
fi
