#!/bin/bash

sudo killall -9 parity && sleep 5
sudo rm -f /usr/bin/parity /usr/bin/uninstall-parity.sh /usr/bin/ethstore /usr/bin/ethkey /usr/bin/parity-evm
