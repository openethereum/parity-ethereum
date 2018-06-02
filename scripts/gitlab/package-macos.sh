#!/bin/bash

set -e # fail on any error
set -u # treat unset variables as error
OSX_PACKAGE="parity_"$VERSION"_macos_x86_64.pkg"
echo "__________Create MacOS package__________"
cd mac
xcodebuild -configuration Release
cd ..
packagesbuild -v mac/Parity.pkgproj
echo "__________Sign Package__________"
find . -name \*.pkg
productsign --sign 'Developer ID Installer: PARITY TECHNOLOGIES LIMITED (P2PX3JU8FT)' Parity\ Ethereum.pkg $OSX_PACKAGE
echo "__________Move package to artifacts__________"
mkdir -p packages
mv -v $OSX_PACKAGE packages/$OSX_PACKAGE
cd packages
echo "_____ Calculating checksums _____"
rhash --sha256 "parity_"$VERSION"_macos_x86_64.pkg" >> "parity_"$VERSION"_macos_x86_64.pkg.sha256"
