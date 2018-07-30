#!/bin/sh
echo Building dapowerplay/parity:build-latest

docker build --no-cache -t dapowerplay/parity:build-latest . -f build.Dockerfile

docker create --name extract dapowerplay/parity:build-latest 
mkdir parity

docker cp extract:/build/parity-ethereum/target/release/parity ./parity
docker rm -f extract

echo Building dapowerplay/parity:latest

docker build --no-cache -t dapowerplay/parity:latest .
rm -rf ./parity
