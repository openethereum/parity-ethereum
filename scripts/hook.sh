#!/bin/sh
FILE=./.git/hooks/pre-push

echo "#!/bin/sh\n" > $FILE
# Exit on any error
echo "set -e" >> $FILE
# Run release build
echo "cargo build --features dev" >> $FILE
# Build tests
echo "cargo test --no-run --features dev --all --exclude parity-ipfs-api --exclude evmjit" >> $FILE
echo "" >> $FILE
chmod +x $FILE
