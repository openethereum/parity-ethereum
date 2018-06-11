#!/bin/sh
FILE=./.git/hooks/pre-push

echo "#!/bin/sh\n" > $FILE
# Exit on any error
echo "set -e" >> $FILE
# Run release build
echo "cargo build -j 8 --features dev" >> $FILE
# Build tests
echo "cargo test -j 8 --no-run --features dev --all --exclude parity-ipfs-api" >> $FILE
echo "" >> $FILE
chmod +x $FILE
