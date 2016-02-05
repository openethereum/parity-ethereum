if ! type kcov > /dev/null; then
   	echo "Install kcov first (details inside this file). Aborting."
	exit 1
fi

cargo test --no-run || exit $?
mkdir -p target/coverage
kcov --exclude-pattern ~/.multirust,rocksdb,secp256k1,sync/src/tests --include-pattern sync/src --verify target/coverage target/debug/ethsync*
xdg-open target/coverage/index.html
