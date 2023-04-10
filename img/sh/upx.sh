#!/usr/bin/env bash

DIR=$(realpath $0) && DIR=${DIR%/*/*}
cd $DIR
set -ex

upx_version=4.0.2

arch=$(uname -m)
case $arch in
  x86_64)
    arch=amd64
    ;;
  aarch64)
    arch=arm64
    ;;
esac

upxdir=upx-$upx_version-${arch}_linux

wget https://github.com/upx/upx/releases/download/v$upx_version/$upxdir.tar.xz -O upx.txz

tar xfJ upx.txz

./$upxdir/upx --best --lzma target/app

#rm -rf $upxdir
