#!/bin/bash
rm -rf parity-import-tests/
cargo build -j $(nproc) --release --features final 
git clone https://github.com/paritytech/parity-import-tests
cp target/release/parity parity-import-tests/aura/parity
cd parity-import-tests/aura
./parity -v
echo "Start Aura test"
./parity import blocks.rlp --chain chain.json
if [ $? -eq 0 ]
then
  echo "Import test passed"
else
  echo "Import test failed" >&2
  exit 1
fi
./parity restore snap --chain chain.json --log-file restore.log
if [ $? -eq 0 ]
then
  echo "Restore test passed"
else
  echo "Restore test failed" >&2
  exit 1
fi
echo "Aura test complete"
