#!/bin/bash
rm -rf parity-import-tests/
cargo build -j $(nproc) --release
git clone https://github.com/paritytech/parity-import-tests
target/release/parity -v
echo "Start Aura test"
target/release/parity import parity-import-tests/aura/blocks.rlp --chain parity-import-tests/aura/chain.json
if [ $? -eq 0 ]
then
  echo "Import test passed"
else
  echo "Import test failed" >&2
  exit 1
fi
target/release/parity restore parity-import-tests/aura/snap --chain parity-import-tests/aura/chain.json
if [ $? -eq 0 ]
then
  echo "Restore test passed"
else
  echo "Restore test failed" >&2
  exit 1
fi
echo "Aura test complete"
