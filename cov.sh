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
if ! type kcov > /dev/null; then
   	echo "Install kcov first (details inside this file). Aborting."
	exit 1
fi

cargo test \
	-p ethash \
	-p ethcore-util \
	-p ethcore \
	-p ethsync \
	-p ethcore-rpc \
	-p parity \
	-p ethminer \
	-p ethcore-signer \
	-p ethcore-dapps \
	--no-run || exit $?
rm -rf target/coverage
mkdir -p target/coverage

EXCLUDE="~/.multirust,rocksdb,secp256k1,src/tests,util/json-tests,util/src/network/tests,sync/src/tests,ethcore/src/tests,ethcore/src/evm/tests"
kcov --exclude-pattern $EXCLUDE --include-pattern src --verify target/coverage target/debug/deps/ethcore-*
kcov --exclude-pattern $EXCLUDE --include-pattern src --verify target/coverage target/debug/deps/ethash-*
kcov --exclude-pattern $EXCLUDE --include-pattern src --verify target/coverage target/debug/deps/ethcore_util-*
kcov --exclude-pattern $EXCLUDE --include-pattern src --verify target/coverage target/debug/deps/ethsync-*
kcov --exclude-pattern $EXCLUDE --include-pattern src --verify target/coverage target/debug/deps/ethcore_rpc-*
kcov --exclude-pattern $EXCLUDE --include-pattern src --verify target/coverage target/debug/deps/ethcore_signer-*
kcov --exclude-pattern $EXCLUDE --include-pattern src --verify target/coverage target/debug/deps/ethcore_dapps-*
kcov --exclude-pattern $EXCLUDE --include-pattern src --verify target/coverage target/debug/deps/ethminer-*
xdg-open target/coverage/index.html
