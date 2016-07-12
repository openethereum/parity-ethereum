#!/bin/sh
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

KCOV=${1:-kcov}

if ! type $KCOV > /dev/null; then
   	echo "Install kcov first (details inside this file). Aborting."
	exit 1
fi

. ./scripts/targets.sh

cargo test $TARGETS --no-run || exit $?
rm -rf target/kcov
mkdir -p target/kcov

EXCLUDE="~/.cargo,~/.multirust,rocksdb,secp256k1,src/tests,util/json-tests,util/src/network/tests,sync/src/tests,ethcore/src/tests,ethcore/src/evm/tests,ethstore/tests"
$KCOV --exclude-pattern $EXCLUDE --include-pattern src target/kcov target/debug/deps/ethkey-*
$KCOV --exclude-pattern $EXCLUDE --include-pattern src target/kcov target/debug/deps/ethstore-*
$KCOV --exclude-pattern $EXCLUDE --include-pattern src target/kcov target/debug/deps/ethcore-*
$KCOV --exclude-pattern $EXCLUDE --include-pattern src target/kcov target/debug/deps/ethash-*
$KCOV --exclude-pattern $EXCLUDE --include-pattern src target/kcov target/debug/deps/ethcore_util-*
$KCOV --exclude-pattern $EXCLUDE --include-pattern src target/kcov target/debug/deps/ethsync-*
$KCOV --exclude-pattern $EXCLUDE --include-pattern src target/kcov target/debug/deps/ethcore_rpc-*
$KCOV --exclude-pattern $EXCLUDE --include-pattern src target/kcov target/debug/deps/ethcore_signer-*
$KCOV --exclude-pattern $EXCLUDE --include-pattern src target/kcov target/debug/deps/ethcore_dapps-*
$KCOV --coveralls-id=${TRAVIS_JOB_ID} --exclude-pattern $EXCLUDE target/kcov target/debug/parity-*

exit 0
