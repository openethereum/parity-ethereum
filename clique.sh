#!/bin/bash

set -e

sudo add-apt-repository -y ppa:ethereum/ethereum
sudo apt install geth

git clone https://github.com/jwasinger/parity-deploy -b clique /home/travis/build/goerli/parity-deploy
ln -s /home/travis/build/goerli/parity-goerli /home/travis/build/goerli/parity-ethereum

cd /home/travis/build/goerli/parity-goerli
cargo build --features final
sudo install -C ./target/debug/parity /usr/local/bin/parity

cd /home/travis/build/goerli/parity-deploy
rm -f docker/parity
cp ../parity-goerli/target/debug/parity ./docker/parity
docker build -t parity/clique ./docker

./parity-deploy.sh -c clique -n 2 -gn 2

sed -i '/volumes\:/d' ./docker-compose.yaml
sed -i '/myapp\:/d' ./docker-compose.yaml
sed -i '/\#driver\:\ local/d' ./docker-compose.yaml
sed -i '/driver\_opts\:/d' ./docker-compose.yaml
sed -i '/o\:\ uid\=1000\,gid\=1000/d' ./docker-compose.yaml

docker-compose up

TIMEOUT=$(echo 50*60-60-$SECONDS|bc)
sleep $TIMEOUT

./clean.sh
