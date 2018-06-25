#!/usr/bin/env bash
# Installing KCOV under ubuntu
# https://users.rust-lang.org/t/tutorial-how-to-collect-test-coverages-for-rust-project/650#
### Install deps
# $ sudo apt-get install libcurl4-openssl-dev libelf-dev libdw-dev cmake gcc binutils-dev libiberty-dev
#
### Compile kcov
# $ wget https://github.com/SimonKagstrom/kcov/archive/master.tar.gz && tar xf master.tar.gz
# $ cd kcov-master && mkdir build && cd build
# $ cmake .. && make && sudo make install

### Running coverage

set -x
RUSTFLAGS="-C link-dead-code" cargo test --all --no-run || exit $?
KCOV_TARGET="target/cov"
KCOV_FLAGS="--verify"
EXCLUDE="/usr/lib,/usr/include,$HOME/.cargo,$HOME/.multirust,rocksdb,secp256k1"
mkdir -p $KCOV_TARGET
echo "Cover RUST"
for FILE in `find target/debug/deps ! -name "*.*"`
  do
   timeout --signal=SIGKILL 5m kcov --exclude-pattern $EXCLUDE $KCOV_FLAGS $KCOV_TARGET $FILE
  done
timeout --signal=SIGKILL 5m kcov --exclude-pattern $EXCLUDE $KCOV_FLAGS $KCOV_TARGET target/debug/parity-*
echo "Cover JS"
cd js
npm install&&npm run test:coverage
cd ..
bash <(curl -s https://codecov.io/bash)&&
echo "Uploaded code coverage"
exit 0
