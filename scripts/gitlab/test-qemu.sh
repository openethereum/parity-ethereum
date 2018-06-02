#!/bin/bash
##ARGUMENTS: 1. Docker target
set -e # fail on any error
set -u # treat unset variables as error

docker pull parity/rust-qemu:armhf
echo "Test wasmi on armhf"
docker run -it --rm -v /usr/bin/qemu-arm-static:/usr/bin/qemu-arm-static parity/rust-qemu:armhf \
bash `git clone https://github.com/paritytech/parity.git&&\
cd parity&&./test.sh&&cd ..&&rm -rf parity/`
