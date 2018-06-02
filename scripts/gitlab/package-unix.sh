#!/bin/bash

set -e # fail on any error
set -u # treat unset variables as error

rm -rf /install
mkdir -p packages
echo "__________Create "$PKG" package__________"
PACKAGE="parity_"$VERSION"_"$IDENT"_"$BUILD_ARCH"."$PKG
mkdir -p /install/usr/bin
cp artifacts/parity /install/usr/bin
cp artifacts/parity-evm /install/usr/bin/parity-evm
cp artifacts/ethstore /install/usr/bin/ethstore
cp artifacts/ethkey /install/usr/bin/ethkey
cp scripts/gitlab/uninstall-parity.sh /install/usr/bin/uninstall-parity.sh
fpm --input-type dir \
--output-type $PKG \
--name parity \
--version $VERSION \
--license GPLv3 \
--depends "$LIBSSL" \
--provides parity \
--url https://parity.io \
--vendor "Parity Technologies" \
--architecture $BUILD_ARCH \
--maintainer "<devops@parity.io>" \
--description "Ethereum network client by Parity Technologies" \
--before-install scripts/gitlab/install-readme.sh \
--before-upgrade scripts/gitlab/uninstall-parity.sh \
--after-remove scripts/gitlab/uninstall-parity.sh \
-C /install \
-p packages/$PACKAGE
echo "_____ Calculating checksums _____"
cd packages
rhash --sha256 $PACKAGE > $PACKAGE".sha256"
