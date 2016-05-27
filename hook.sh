#!/bin/sh
FILE=./.git/hooks/pre-push
echo "#!/bin/sh\n" > $FILE
# Exit on any error
echo "set -e" >> $FILE
# Run release build
echo "cargo build --features dev" >> $FILE
# Build tests
echo "cargo test --no-run --features dev \\" >> $FILE
echo "	-p ethash -p ethcore-util -p ethcore -p ethsync -p ethcore-rpc -p parity -p ethminer -p ethcore-dapps -p ethcore-signer" >> $FILE
echo "" >> $FILE
chmod +x $FILE
