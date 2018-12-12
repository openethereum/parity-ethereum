#!/bin/bash

set -e

cd $TRAVIS_BUILD_DIR/..
git clone https://github.com/jwasinger/parity-deploy -b clique
ln -s $TRAVIS_BUILD_DIR/parity-goerli $TRAVIS_BUILD_DIR/parity-ethereum
cd parity-deploy
./build.sh
./parity-deploy.sh -c clique -n 2 -gn 2

sed -i '/volumes\:/d' docker-compose.yaml
sed -i '/myapp\:/d' docker-compose.yaml
sed -i '/\#driver\:\ local/d' docker-compose.yaml
sed -i '/driver\_opts\:/d' docker-compose.yaml
sed -i '/o\:\ uid\=1000\,gid\=1000/d' docker-compose.yaml

docker-compose up

TIMEOUT=$(echo 50*60-60-$SECONDS|bc)
sleep $TIMEOUT

./clean.sh
