#!/bin/bash

set -e

git clone https://github.com/jwasinger/parity-deploy -b clique
ln -s $TRAVIS_BUILD_DIR parity-ethereum
cd parity-deploy
./build.sh
./parity-deploy.sh -c clique -n 2 -gn 2
