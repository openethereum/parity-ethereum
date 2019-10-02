#!/bin/bash
set -e # fail on any error
set -u # treat unset variables as error
echo "________Running validate_chainspecs.sh________"

ERR=0

echo "________Validate chainspecs________"
time cargo build --release -p chainspec --verbose --color=always

for spec in ethcore/res/*.json; do
    if ! ./target/release/chainspec "$spec"; then ERR=1; fi
done

for spec in ethcore/res/ethereum/*.json; do
    if ! ./target/release/chainspec "$spec"; then ERR=1; fi
done
#show sccache statistics
sccache --stop-server
exit $ERR
