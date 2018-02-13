#!/bin/bash
set -e # fail on any error
set -u # treat unset variables as error

cargo build -j $(nproc) --release --features final $CARGOFLAGS
git clone https://github.com/paritytech/parity-import-tests
cp target/release/parity parity-import-tests/aura/parity
cd parity-import-tests/aura
echo "Start Aura test"
parity import blocks.rlp --chain chain.json
parity restore snap --chain chain.json
echo "Aura test complete"
