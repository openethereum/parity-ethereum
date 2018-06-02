#!/bin/bash
set -e # fail on any error
set -u # treat unset variables as error

VERSION=$(grep -m 1 "version =" Cargo.toml | awk '{print $3}' | tr -d '"' | tr -d "\n")
echo "__________Create Windows package__________"
scripts/gitlab/msbuild.cmd
echo "__________Sign binaries__________"
scripts/gitlab/sign.cmd $keyfile $certpass artifacts/parity.exe
scripts/gitlab/sign.cmd $keyfile $certpass windows/ptray/x64/release/ptray.exe
echo "__________Create Windows installer__________"
cd nsis
curl -sL --url "https://github.com/paritytech/win-build/raw/master/vc_redist.x64.exe" -o vc_redist.x64.exe
echo "makensis.exe installer.nsi" > nsis.cmd
./nsis.cmd
cd ..
echo "__________Move package to artifacts__________"
mkdir -p packages
cp nsis/installer.exe packages/"parity_"$VERSION"_windows_x86_64.exe"
echo "__________Sign installer__________"
scripts/gitlab/sign.cmd $keyfile $certpass packages/"parity_"$VERSION"_windows_x86_64.exe"
echo "_____ Calculating checksums _____"
cd packages
for binary in $(ls)
do
  rhash --sha256 $binary -o $binary.sha256
done
