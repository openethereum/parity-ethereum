#!/bin/bash

set -e

echo "________________________________________________________________________________"
echo "INSTALL GETH: sudo apt install ethereum"
sudo add-apt-repository -y ppa:ethereum/ethereum
sudo apt update
sudo apt install -y ethereum

echo "________________________________________________________________________________"
echo "BUILD PARITY: cargo build --features final"
git clone https://github.com/jwasinger/parity-deploy -b clique /home/travis/build/goerli/parity-deploy
ln -s /home/travis/build/goerli/parity-goerli /home/travis/build/goerli/parity-ethereum
cd /home/travis/build/goerli/parity-goerli
cargo build --features final
cargo build -p ethkey-cli

echo "________________________________________________________________________________"
echo "INSTALL PARITY: sudo install -C parity"
sudo install -C ./target/debug/parity /usr/local/bin/parity
sudo install -C ./target/debug/ethkey /usr/local/bin/ethkey

echo "________________________________________________________________________________"
echo "BUILD DOCKER IMAGE: docker build -t parity/clique"
cd /home/travis/build/goerli/parity-deploy
git pull --rebase https://github.com/paritytech/parity-deploy master
rm -f docker/parity
cp ../parity-goerli/target/debug/parity ./docker/parity
docker build -t parity/clique ./docker

echo "________________________________________________________________________________"
echo "DEPLOY DOCKER IMAGE: --config clique --nodes 2 --geth-nodes 2"
./parity-deploy.sh --config clique --nodes 2 --geth-nodes 2
TIMEOUT=$(echo 50*60-600-$SECONDS|bc)

echo "________________________________________________________________________________"
echo "RUN NETWORK FOR $TIMEOUT SECONDS: docker-compose up"
docker-compose up & sleep $TIMEOUT && docker-compose stop
sudo docker-compose rm -f
sudo ./clean.sh
