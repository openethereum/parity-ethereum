#!/bin/sh

echo "#################################################################################"
echo "This image is deprecated and will not be maintained on hub.docker.com in a while."
echo "                Please use 'parity/ethereum' image instead."
echo "#################################################################################"
./bin/parity $@
