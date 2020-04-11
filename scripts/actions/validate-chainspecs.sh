#!/bin/bash
set -e # fail on any error
set -u # treat unset variables as error
echo -e "\n*** Running \`validate_chainspecs.sh\`"
ERR=0
time cargo build --release -p chainspec --verbose --color=always

echo -e "\n*** Validating custom chain specifications:"
for spec in ethcore/res/*.json; do
    if ! ./target/release/chainspec "$spec"; then ERR=1; fi
done

echo -e "\n*** Validating test-chain specifications:"
for spec in ethcore/res/ethereum/test-specs/*.json; do
    if ! ./target/release/chainspec "$spec"; then ERR=1; fi
done

echo -e "\n*** Validating ethereum chain specifications:"
for spec in ethcore/res/ethereum/*.json; do
    if ! ./target/release/chainspec "$spec"; then ERR=1; fi
done

echo -e "\n*** Checking mainnet EIPs against test specifications:"
for spec in "ethcore/res/ethereum/foundation.json" "ethcore/res/ethereum/classic.json"; do
    for fork in "frontier" "homestead" "byzantium" "constantinople" "st_peters" "istanbul"; do
        for eip in $(grep --only-matching "eip.*Transition" ethcore/res/ethereum/test-specs/${fork}_test.json); do
            if ! grep -q $eip $spec; then
                echo "ERROR: $fork $eip is missing in the $spec"
                ERR=1
            else
                echo "$spec contains $fork $eip"
            fi
        done
    done
done

exit $ERR
