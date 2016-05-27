#!/bin/sh
# Running Parity Full Test Sute

cargo test --features ethcore/json-tests $1 \
	-p ethash \
	-p ethcore-util \
	-p ethcore \
	-p ethsync \
	-p ethcore-rpc \
	-p ethcore-signer \
	-p ethcore-dapps \
	-p parity \
	-p ethminer \
	-p bigint
