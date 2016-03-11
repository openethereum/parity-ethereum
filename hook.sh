#!/bin/sh
FILE=./.git/hooks/pre-push
echo "#!/bin/sh\n" > $FILE
# Exit on any error
echo "set -e" >> $FILE
# Run release build
echo "cargo build --release --features dev-clippy" >> $FILE
# Build tests
echo "cargo test --no-run --features dev-clippy \\" >> $FILE
echo "	-p ethash -p ethcore-util -p ethcore -p ethsync -p ethcore-rpc -p parity" >> $FILE
echo "" >> $FILE
chmod +x $FILE
