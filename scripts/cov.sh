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

KCOV_CMD=${1:-kcov}

if ! type $KCOV_CMD > /dev/null; then
   	echo "Install kcov first (details inside this file). Aborting."
	exit 1
fi

. ./scripts/targets.sh

cargo test $TARGETS --no-run || exit $?
rm -rf target/coverage
mkdir -p target/coverage

EXCLUDE="~/.multirust,rocksdb,secp256k1,/usr/,/.cargo,/root/.multirust,src/tests,util/json-tests,util/src/network/tests,sync/src/tests,ethcore/src/tests,ethcore/src/evm/tests,ethstore/tests target/kcov"
$KCOV_CMD --exclude-pattern $EXCLUDE --include-pattern src --verify target/coverage target/debug/deps/ethkey-*
$KCOV_CMD --exclude-pattern $EXCLUDE --include-pattern src --verify target/coverage target/debug/deps/ethstore-*
$KCOV_CMD --exclude-pattern $EXCLUDE --include-pattern src --verify target/coverage target/debug/deps/ethcore-*
$KCOV_CMD --exclude-pattern $EXCLUDE --include-pattern src --verify target/coverage target/debug/deps/ethash-*
$KCOV_CMD --exclude-pattern $EXCLUDE --include-pattern src --verify target/coverage target/debug/deps/ethcore_util-*
$KCOV_CMD --exclude-pattern $EXCLUDE --include-pattern src --verify target/coverage target/debug/deps/ethsync-*
$KCOV_CMD --exclude-pattern $EXCLUDE --include-pattern src --verify target/coverage target/debug/deps/ethcore_rpc-*
$KCOV_CMD --exclude-pattern $EXCLUDE --include-pattern src --verify target/coverage target/debug/deps/ethcore_signer-*
$KCOV_CMD --exclude-pattern $EXCLUDE --include-pattern src --verify target/coverage target/debug/deps/ethcore_dapps-*
$KCOV_CMD --exclude-pattern $EXCLUDE --include-pattern src --verify target/coverage target/debug/deps/ethjson-*
$KCOV_CMD --exclude-pattern $EXCLUDE --include-pattern src --verify target/coverage target/debug/parity-*
xdg-open target/coverage/index.html
