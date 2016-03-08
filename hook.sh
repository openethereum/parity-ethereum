#!/bin/sh
echo "#!/bin/sh\ncargo test -p ethash -p ethcore-util -p ethcore -p ethsync -p ethcore-rpc -p parity -p ethminer --features dev" > ./.git/hooks/pre-push
chmod +x ./.git/hooks/pre-push
