#!/bin/bash
# uninstall any ancient version
test -f /usr/local/libexec/uninstall-parity.sh && /usr/local/libexec/uninstall-parity.sh || true
killall -9 parity && sleep 5
su $USER -c "open /Applications/Parity\ Ethereum.app"
exit 0
