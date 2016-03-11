#!/bin/sh
echo "#!/bin/sh\ncargo build --features dev-clippy && cargo test --no-run -p ethash -p ethcore-util -p ethcore -p ethsync -p ethcore-rpc -p parity -p ethminer --features dev-clippy" > ./.git/hooks/pre-push
chmod +x ./.git/hooks/pre-push
