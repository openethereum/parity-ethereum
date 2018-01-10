#!/bin/bash

set -e # fail on any error
set -u # treat unset variables as error
# ARGUMENTS: 1. BUILD_PLATFORM (target for binaries) 2. PLATFORM (target for cargo) 3. & 4. CC & CXX flags
# 5. ARC (architecture) 6.EXT (package extention)
BUILD_PLATFORM=$1
PLATFORM=$2
ARC=$3
CC=$4
CXX=$5
EXT=deb
VER=$(grep -m 1 version Cargo.toml | awk '{print $3}' | tr -d '"' | tr -d "\n")
echo "--------------------"
echo "Build for platform: " $BUILD_PLATFORM
echo "Cargo target:       " $PLATFORM
echo "CC&CXX flags:       " $CC ", " $CXX
echo "Architecture:       " $ARC
echo "Libssl version:     " $LIBSSL
echo "Package:            " $EXT
echo "Parity version:     " $VER
echo "Branch:             " $CI_BUILD_REF_NAME
echo "--------------------"
echo "RUST:" rustup show
echo "Cargo:" cargo -V
echo "NODEJS:" nodejs -v
echo "NPM:" npm -v

set_env () {
  echo "Set ENVIROMENT"
  HOST_CC=gcc
  HOST_CXX=g++
  rm -rf .cargo
  mkdir -p .cargo
  echo "[target.$PLATFORM]" >> .cargo/config
  echo "linker= \"$CC\"" >> .cargo/config
  cat .cargo/config
}
build () {
  echo "Build parity:"
  cargo build --target $PLATFORM --features final --release $CARGOFLAGS
  echo "Build evmbin:"
  cargo build --target $PLATFORM --release -p evmbin
  echo "Build ethstore-cli:"
  cargo build --target $PLATFORM --release -p ethstore-cli
  echo "Build ethkep-cli:"
  cargo build --target $PLATFORM --release -p ethkey-cli
  echo "Strip binaries:"
  $STRIP_BIN -v target/$PLATFORM/release/parity
  $STRIP_BIN -v target/$PLATFORM/release/parity-evm
  $STRIP_BIN -v target/$PLATFORM/release/ethstore
  $STRIP_BIN -v target/$PLATFORM/release/ethkey
  echo "Checksum calculation:"
  rm -rf *.md5
  export SHA3=$(target/$PLATFORM/release/parity tools hash target/$PLATFORM/release/parity)
  echo "Parity file SHA3:" $SHA3
  md5sum target/$PLATFORM/release/parity > parity.md5
  md5sum target/$PLATFORM/release/parity-evm > parity-evm.md5
  md5sum target/$PLATFORM/release/ethstore > ethstore.md5
  md5sum target/$PLATFORM/release/ethkey > ethkey.md5
}
make_deb () {
  rm -rf deb
  echo "create DEBIAN files"
  mkdir -p deb/usr/bin/
  mkdir -p deb/DEBIAN
  echo "create copyright, docs, compat"
  cp LICENSE deb/DEBIAN/copyright
  echo "https://github.com/paritytech/parity/wiki" >> deb/DEBIAN/docs
  echo "8" >> deb/DEBIAN/compat
  echo "create control file"
  control=deb/DEBIAN/control
  echo "Package: parity" >> $control
  echo "Version: $VER" >> $control
  echo "Source: parity" >> $control
  echo "Section: science" >> $control
  echo "Priority: extra" >> $control
  echo "Maintainer: Parity Technologies <devops@parity.io>" >> $control
  echo "Build-Depends: debhelper (>=9)" >> $control
  echo "Standards-Version: 3.9.5" >> $control
  echo "Homepage: https://parity.io" >> $control
  echo "Vcs-Git: git://github.com/paritytech/parity.git" >> $control
  echo "Vcs-Browser: https://github.com/paritytech/parity" >> $control
  echo "Architecture: $ARC" >> $control
  echo "Depends: $LIBSSL" >> $control
  echo "Description: Ethereum network client by Parity Technologies" >> $control
  size=`du deb/|awk 'END {print $1}'`
  echo "Installed-Size: $size" >> $control
  echo "build .deb package"
  cp target/$PLATFORM/release/parity deb/usr/bin/parity
  cp target/$PLATFORM/release/parity-evm deb/usr/bin/parity-evm
  cp target/$PLATFORM/release/ethstore deb/usr/bin/ethstore
  cp target/$PLATFORM/release/ethkey deb/usr/bin/ethkey
  dpkg-deb -b deb "parity_"$VER"_"$ARC".deb"
  md5sum "parity_"$VER"_"$ARC".deb" > "parity_"$VER"_"$ARC".deb.md5"
}
make_pkg () {
  echo "make PKG"
  cd mac
  xcodebuild -configuration Release
  cd ..
  packagesbuild -v mac/Parity.pkgproj
  productsign --sign 'Developer ID Installer: PARITY TECHNOLOGIES LIMITED (P2PX3JU8FT)' target/$PLATFORM/release/Parity\ Ethereum.pkg target/$PLATFORM/release/Parity\ Ethereum-signed.pkg
  mv target/$PLATFORM/release/Parity\ Ethereum-signed.pkg "parity-"$VER"_"$ARC".pkg"
  md5sum "parity-"$VER"_"$ARC"."$EXT >> "parity-"$VER"_"$ARC".pkg.md5"
}
push_binaries () {
  echo "Push binaries to AWS S3"
  aws configure set aws_access_key_id $s3_key
  aws configure set aws_secret_access_key $s3_secret
  if [[ "$CI_BUILD_REF_NAME" = "master" || "$CI_BUILD_REF_NAME" = "beta" || "$CI_BUILD_REF_NAME" = "stable" || "$CI_BUILD_REF_NAME" = "nightly" ]];
  then
    export S3_BUCKET=builds-parity-published;
  else
    export S3_BUCKET=builds-parity;
  fi
  aws s3 rm --recursive s3://$S3_BUCKET/$CI_BUILD_REF_NAME/$BUILD_PLATFORM
  aws s3api put-object --bucket $S3_BUCKET --key $CI_BUILD_REF_NAME/$BUILD_PLATFORM/parity --body target/$PLATFORM/release/parity
  aws s3api put-object --bucket $S3_BUCKET --key $CI_BUILD_REF_NAME/$BUILD_PLATFORM/parity.md5 --body parity.md5
  aws s3api put-object --bucket $S3_BUCKET --key $CI_BUILD_REF_NAME/$BUILD_PLATFORM/parity-evm --body target/$PLATFORM/release/parity-evm
  aws s3api put-object --bucket $S3_BUCKET --key $CI_BUILD_REF_NAME/$BUILD_PLATFORM/parity-evm.md5 --body parity-evm.md5
  aws s3api put-object --bucket $S3_BUCKET --key $CI_BUILD_REF_NAME/$BUILD_PLATFORM/ethstore --body target/$PLATFORM/release/ethstore
  aws s3api put-object --bucket $S3_BUCKET --key $CI_BUILD_REF_NAME/$BUILD_PLATFORM/ethstore.md5 --body ethstore.md5
  aws s3api put-object --bucket $S3_BUCKET --key $CI_BUILD_REF_NAME/$BUILD_PLATFORM/ethkey --body target/$PLATFORM/release/ethkey
  aws s3api put-object --bucket $S3_BUCKET --key $CI_BUILD_REF_NAME/$BUILD_PLATFORM/ethkey.md5 --body ethkey.md5
  aws s3api put-object --bucket $S3_BUCKET --key $CI_BUILD_REF_NAME/$BUILD_PLATFORM/"parity_"$VER"_"$ARC"."$EXT --body "parity_"$VER"_"$ARC"."$EXT
  aws s3api put-object --bucket $S3_BUCKET --key $CI_BUILD_REF_NAME/$BUILD_PLATFORM/"parity_"$VER"_"$ARC"."$EXT".md5" --body "parity_"$VER"_"$ARC"."$EXT".md5"
}
make_archive () {
  echo "add artifacts to archive"
  rm -rf parity.zip
  zip -r parity.zip target/$PLATFORM/release/parity target/$PLATFORM/release/parity-evm target/$PLATFORM/release/ethstore target/$PLATFORM/release/ethkey parity.md5 parity-evm.md5 ethstore.md5 ethkey.md5
}
push_release () {
  echo "push release"
  curl --data "commit=$CI_BUILD_REF&sha3=$SHA3&filename=parity&secret=$RELEASES_SECRET" http://update.parity.io:1337/push-build/$CI_BUILD_REF_NAME/$PLATFORM
  curl --data "commit=$CI_BUILD_REF&sha3=$SHA3&filename=parity&secret=$RELEASES_SECRET" http://update.parity.io:1338/push-build/$CI_BUILD_REF_NAME/$PLATFORM
}
windows () {
  set PLATFORM=x86_64-pc-windows-msvc
  set INCLUDE="C:\Program Files (x86)\Microsoft SDKs\Windows\v7.1A\Include;C:\vs2015\VC\include;C:\Program Files (x86)\Windows Kits\10\Include\10.0.10240.0\ucrt"
  set LIB="C:\vs2015\VC\lib;C:\Program Files (x86)\Windows Kits\10\Lib\10.0.10240.0\ucrt\x64"
  set RUST_BACKTRACE=1
  set RUSTFLAGS=%RUSTFLAGS%
  rustup default stable-x86_64-pc-windows-msvc
  cargo clean
  cargo build --features final --release #%CARGOFLAGS%
  cargo build --release -p ethstore-cli #%CARGOFLAGS%
  cargo build --release -p ethkey-cli #%CARGOFLAGS%
  cargo build --release -p evmbin #%CARGOFLAGS%
  signtool sign /f %keyfile% /p %certpass% target\release\parity.exe
  target\release\parity.exe tools hash target\release\parity.exe > parity.sha3
  set /P SHA3=<parity.sha3
  curl -sL --url "https://github.com/paritytech/win-build/raw/master/SimpleFC.dll" -o nsis\SimpleFC.dll
  curl -sL --url "https://github.com/paritytech/win-build/raw/master/vc_redist.x64.exe" -o nsis\vc_redist.x64.exe
  msbuild windows\ptray\ptray.vcxproj /p:Platform=x64 /p:Configuration=Release
  signtool sign /f %keyfile% /p %certpass% windows\ptray\x64\release\ptray.exe
  cd nsis
  makensis.exe installer.nsi
  copy installer.exe InstallParity.exe
  signtool sign /f %keyfile% /p %certpass% InstallParity.exe
  md5sums InstallParity.exe > InstallParity.exe.md5
  zip win-installer.zip InstallParity.exe InstallParity.exe.md5
  md5sums win-installer.zip > win-installer.zip.md5
  cd ..\target\release\
  md5sums parity.exe > parity.exe.md5
  zip parity.zip parity.exe parity.md5
  md5sums parity.zip > parity.zip.md5
  cd ..\..
  aws configure set aws_access_key_id %s3_key%
  aws configure set aws_secret_access_key %s3_secret%
  echo %CI_BUILD_REF_NAME%
  echo %CI_BUILD_REF_NAME% | findstr /R "master" >nul 2>&1 && set S3_BUCKET=builds-parity-published|| set S3_BUCKET=builds-parity
  echo %CI_BUILD_REF_NAME% | findstr /R "beta" >nul 2>&1 && set S3_BUCKET=builds-parity-published|| set S3_BUCKET=builds-parity
  echo %CI_BUILD_REF_NAME% | findstr /R "stable" >nul 2>&1 && set S3_BUCKET=builds-parity-published|| set S3_BUCKET=builds-parity
  echo %CI_BUILD_REF_NAME% | findstr /R "nightly" >nul 2>&1 && set S3_BUCKET=builds-parity-published|| set S3_BUCKET=builds-parity
  echo %S3_BUCKET%
  aws s3 rm --recursive s3://%S3_BUCKET%/%CI_BUILD_REF_NAME%/x86_64-pc-windows-msvc
  aws s3api put-object --bucket %S3_BUCKET% --key %CI_BUILD_REF_NAME%/x86_64-pc-windows-msvc/parity.exe --body target\release\parity.exe
  aws s3api put-object --bucket %S3_BUCKET% --key %CI_BUILD_REF_NAME%/x86_64-pc-windows-msvc/parity.exe.md5 --body target\release\parity.exe.md5
  aws s3api put-object --bucket %S3_BUCKET% --key %CI_BUILD_REF_NAME%/x86_64-pc-windows-msvc/parity.zip --body target\release\parity.zip
  aws s3api put-object --bucket %S3_BUCKET% --key %CI_BUILD_REF_NAME%/x86_64-pc-windows-msvc/parity.zip.md5 --body target\release\parity.zip.md5
  aws s3api put-object --bucket %S3_BUCKET% --key %CI_BUILD_REF_NAME%/x86_64-pc-windows-msvc/InstallParity.exe --body nsis\InstallParity.exe
  aws s3api put-object --bucket %S3_BUCKET% --key %CI_BUILD_REF_NAME%/x86_64-pc-windows-msvc/InstallParity.exe.md5 --body nsis\InstallParity.exe.md5
  aws s3api put-object --bucket %S3_BUCKET% --key %CI_BUILD_REF_NAME%/x86_64-pc-windows-msvc/win-installer.zip --body nsis\win-installer.zip
  aws s3api put-object --bucket %S3_BUCKET% --key %CI_BUILD_REF_NAME%/x86_64-pc-windows-msvc/win-installer.zip.md5 --body nsis\win-installer.zip.md5
}
case $BUILD_PLATFORM in
  x86_64-unknown-linux-gnu)
    #set strip bin
    STRIP_BIN="strip"
    build
    make_deb
    make_archive
    push_binaries
    push_release
    ;;
  x86_64-unknown-debian-gnu)
    STRIP_BIN="strip"
    LIBSSL: "libssl1.1.0 (>=1.1.0)"
    echo "Use libssl1.1.0 (>=1.1.0) for Debian builds"
    build
    make_deb
    make_archive
    push_binaries
    push_release
    ;;
  x86_64-unknown-centos-gnu)
    STRIP_BIN="strip"
    build
    make_archive
    push_binaries
    push_release
    ;;
  i686-unknown-linux-gnu)
    STRIP_BIN="strip"
    set_env
    build
    make_deb
    make_archive
    push_binaries
    push_release
    ;;
  armv7-unknown-linux-gnueabihf)
    STRIP_BIN="arm-linux-gnueabihf-strip"
    set_env
    build
    make_deb
    make_archive
    push_binaries
    push_release
    ;;
  arm-unknown-linux-gnueabihf)
    STRIP_BIN="arm-linux-gnueabihf-strip"
    set_env
    build
    make_deb
    make_archive
    push_binaries
    push_release
    ;;
  aarch64-unknown-linux-gnu)
    STRIP_BIN="aarch64-linux-gnu-strip"
    set_env
    build
    make_deb
    make_archive
    push_binaries
    push_release
    ;;
  x86_64-apple-darwin)
    STRIP_BIN="strip"
    PLATFORM="x86_64-apple-darwin"
    EXT="pkg"
    build
    make_pkg
    make_archive
    push_binaries
    push_release
    ;;
  x86_64-unknown-snap-gnu)
    cd snap
    ARC="amd64"
    EXT="snap"
    rm -rf *snap
    sed -i 's/master/'"$VER"'/g' snapcraft.yaml
    snapcraft
    cp "parity_"$CI_BUILD_REF_NAME"_amd64.snap" "parity_"$VER"_amd64.snap"
    md5sum "parity_"$VER"_amd64.snap" > "parity_"$VER"_amd64.snap.md5"
    push_binaries
    ;;
  rust_beta)
    rustup default beta
    export STRIP_BIN="strip"
    build
    make_archive
    ;;
  rust_nightly)
    rustup default nightly
    export STRIP_BIN="strip"
    build
    make_archive
    ;;
  x86_64-pc-windows-msvc)
    windows
    push_release
esac
