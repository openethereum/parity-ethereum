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

set -x

KCOV=${1:-kcov}

if ! type $KCOV > /dev/null; then
   	echo "Install kcov first (details inside this file). Aborting."
	exit 1
fi

. ./scripts/targets.sh
cargo test $TARGETS --no-run || exit $?



KCOV_TARGET="target/kcov"
KCOV_FLAGS="--verify"
EXCLUDE="/usr/lib,\
/usr/include,\
$HOME/.cargo,\
$HOME/.multirust,\
rocksdb,\
secp256k1,\
src/tests,\
util/json-tests,\
util/src/network/tests,\
ethcore/src/evm/tests,\
ethstore/tests,\
target/debug/build,\
target/release/build\
"

rm -rf $KCOV_TARGET
mkdir -p $KCOV_TARGET

for FILE in `find target/debug/deps ! -name "*.*"`
do
	$KCOV --exclude-pattern $EXCLUDE $KCOV_FLAGS $KCOV_TARGET $FILE
done

$KCOV --coveralls-id=${CI_BUILD_ID} --exclude-pattern $EXCLUDE $KCOV_FLAGS $KCOV_TARGET target/debug/parity-*

exit 0
