#!/bin/bash

set -e

echo "________________________________________________________________________________"
echo "INSTALL GETH: sudo apt install ethereum"
sudo add-apt-repository -y ppa:ethereum/ethereum
sudo apt install ethereum

echo "________________________________________________________________________________"
echo "BUILD PARITY: cargo build --features final"
git clone https://github.com/jwasinger/parity-deploy -b clique /home/travis/build/goerli/parity-deploy
ln -s /home/travis/build/goerli/parity-goerli /home/travis/build/goerli/parity-ethereum
cd /home/travis/build/goerli/parity-goerli
cargo build --features final

echo "________________________________________________________________________________"
echo "INSTALL PARITY: sudo install -C parity"
sudo install -C ./target/debug/parity /usr/local/bin/parity

echo "________________________________________________________________________________"
echo "BUILD DOCKER IMAGE: docker build -t parity/clique"
cd /home/travis/build/goerli/parity-deploy
rm -f docker/parity
cp ../parity-goerli/target/debug/parity ./docker/parity
docker build -t parity/clique ./docker

echo "________________________________________________________________________________"
echo "DEPLOY DOCKER IMAGE: --config clique --nodes 2 --geth-nodes 2"
./parity-deploy.sh --config clique --nodes 2 --geth-nodes 2
sed -i '/volumes\:/d' ./docker-compose.yaml
sed -i '/myapp\:/d' ./docker-compose.yaml
sed -i '/\#driver\:\ local/d' ./docker-compose.yaml
sed -i '/driver\_opts\:/d' ./docker-compose.yaml
sed -i '/o\:\ uid\=1000\,gid\=1000/d' ./docker-compose.yaml
TIMEOUT=$(echo 50*60-60-$SECONDS|bc)

echo "________________________________________________________________________________"
echo "RUN NETWORK FOR $TIMEOUT SECONDS: docker-compose up"
docker-compose up
sleep $TIMEOUT
./clean.sh
