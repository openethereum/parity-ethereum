#!/usr/bin/env bash

set -e # fail on any error
set -u # treat unset variables as error
#ARGUMENTS: 1. BUILD_PLATFORM (target for binaries) 2. PLATFORM (target for cargo) 3. ARC (architecture) 4. & 5. CC & CXX flags 6. binary identifier
BUILD_PLATFORM=$1
PLATFORM=$2
ARC=$3
CC=$4
CXX=$5
IDENT=$6
VER="$(grep -m 1 "version =" Cargo.toml | awk '{print $3}' | tr -d '"' | tr -d "\n")"
S3WIN=""
echo "--------------------"
echo "Build for platform: " $BUILD_PLATFORM
echo "Build identifier:   " $IDENT
echo "Cargo target:       " $PLATFORM
echo "CC&CXX flags:       " $CC ", " $CXX
echo "Architecture:       " $ARC
echo "Parity version:     " $VER
echo "Branch:             " $CI_BUILD_REF_NAME
echo "--------------------"

# NOTE for sha256 we want to display filename as well
# hence we use --* instead of -p *
SHA256_BIN="rhash --sha256"

set_env () {
  echo "Set ENVIROMENT"
  export HOST_CC=gcc
  export HOST_CXX=g++
  rm -rf .cargo
  mkdir -p .cargo
  echo "[target.$PLATFORM]" >> .cargo/config
  echo "linker= \"$CC\"" >> .cargo/config
  cat .cargo/config
}
set_env_win () {
  set PLATFORM=x86_64-pc-windows-msvc
  set INCLUDE="C:\Program Files (x86)\Microsoft SDKs\Windows\v7.1A\Include;C:\vs2015\VC\include;C:\Program Files (x86)\Windows Kits\10\Include\10.0.10240.0\ucrt"
  set LIB="C:\vs2015\VC\lib;C:\Program Files (x86)\Windows Kits\10\Lib\10.0.10240.0\ucrt\x64"
  set RUST_BACKTRACE=1
  #export RUSTFLAGS=$RUSTFLAGS
  rustup default stable-x86_64-pc-windows-msvc
  echo "@ signtool sign /f "\%"1 /p "\%"2 /tr http://timestamp.comodoca.com /du https://parity.io "\%"3" > sign.cmd
}
build () {
  if [[ "windows" = $IDENT ]]
  then
    # This is a nasty hack till we figure out the proper cargo caching strategy
    echo "Remove index"
    rm -rf cargo/registry/index/*.
  fi
  echo "Build parity:"
  cargo build --target $PLATFORM --features final --release
  echo "Build evmbin:"
  cargo build --target $PLATFORM --release -p evmbin
  echo "Build ethstore-cli:"
  cargo build --target $PLATFORM --release -p ethstore-cli
  echo "Build ethkey-cli:"
  cargo build --target $PLATFORM --release -p ethkey-cli
  echo "Build whisper-cli:"
  cargo build --target $PLATFORM --release -p whisper-cli
}
strip_binaries () {
  echo "Strip binaries:"
  $STRIP_BIN -v target/$PLATFORM/release/parity
  $STRIP_BIN -v target/$PLATFORM/release/parity-evm
  $STRIP_BIN -v target/$PLATFORM/release/ethstore
  $STRIP_BIN -v target/$PLATFORM/release/ethkey
  $STRIP_BIN -v target/$PLATFORM/release/whisper;
}
calculate_checksums () {
  echo "Checksum calculation:"
  rhash --version

  rm -rf *.sha256

  BIN="target/$PLATFORM/release/parity$S3WIN"
  export SHA3="$($BIN tools hash $BIN)"

  echo "Parity file SHA3: $SHA3"
  $SHA256_BIN target/$PLATFORM/release/parity$S3WIN > parity$S3WIN.sha256
  $SHA256_BIN target/$PLATFORM/release/parity-evm$S3WIN > parity-evm$S3WIN.sha256
  $SHA256_BIN target/$PLATFORM/release/ethstore$S3WIN > ethstore$S3WIN.sha256
  $SHA256_BIN target/$PLATFORM/release/ethkey$S3WIN > ethkey$S3WIN.sha256
  $SHA256_BIN target/$PLATFORM/release/whisper$S3WIN > whisper$S3WIN.sha256
}
sign_exe () {
  ./sign.cmd $keyfile $certpass "target/$PLATFORM/release/parity.exe"
  ./sign.cmd $keyfile $certpass "target/$PLATFORM/release/parity-evm.exe"
  ./sign.cmd $keyfile $certpass "target/$PLATFORM/release/ethstore.exe"
  ./sign.cmd $keyfile $certpass "target/$PLATFORM/release/ethkey.exe"
  ./sign.cmd $keyfile $certpass "target/$PLATFORM/release/whisper.exe"
}
push_binaries () {
  echo "Push binaries to AWS S3"
  aws configure set aws_access_key_id $s3_key
  aws configure set aws_secret_access_key $s3_secret
  if [[ "$CI_BUILD_REF_NAME" = "beta" || "$CI_BUILD_REF_NAME" = "stable" || "$CI_BUILD_REF_NAME" = "nightly" ]];
  then
    export S3_BUCKET=builds-parity-published;
  else
    export S3_BUCKET=builds-parity;
  fi
  aws s3 rm --recursive s3://$S3_BUCKET/$CI_BUILD_REF_NAME/$BUILD_PLATFORM
  aws s3api put-object --bucket $S3_BUCKET --key $CI_BUILD_REF_NAME/$BUILD_PLATFORM/parity$S3WIN --body target/$PLATFORM/release/parity$S3WIN
  aws s3api put-object --bucket $S3_BUCKET --key $CI_BUILD_REF_NAME/$BUILD_PLATFORM/parity$S3WIN.sha256 --body parity$S3WIN.sha256
  aws s3api put-object --bucket $S3_BUCKET --key $CI_BUILD_REF_NAME/$BUILD_PLATFORM/parity-evm$S3WIN --body target/$PLATFORM/release/parity-evm$S3WIN
  aws s3api put-object --bucket $S3_BUCKET --key $CI_BUILD_REF_NAME/$BUILD_PLATFORM/parity-evm$S3WIN.sha256 --body parity-evm$S3WIN.sha256
  aws s3api put-object --bucket $S3_BUCKET --key $CI_BUILD_REF_NAME/$BUILD_PLATFORM/ethstore$S3WIN --body target/$PLATFORM/release/ethstore$S3WIN
  aws s3api put-object --bucket $S3_BUCKET --key $CI_BUILD_REF_NAME/$BUILD_PLATFORM/ethstore$S3WIN.sha256 --body ethstore$S3WIN.sha256
  aws s3api put-object --bucket $S3_BUCKET --key $CI_BUILD_REF_NAME/$BUILD_PLATFORM/ethkey$S3WIN --body target/$PLATFORM/release/ethkey$S3WIN
  aws s3api put-object --bucket $S3_BUCKET --key $CI_BUILD_REF_NAME/$BUILD_PLATFORM/ethkey$S3WIN.sha256 --body ethkey$S3WIN.sha256
  aws s3api put-object --bucket $S3_BUCKET --key $CI_BUILD_REF_NAME/$BUILD_PLATFORM/whisper$S3WIN --body target/$PLATFORM/release/whisper$S3WIN
  aws s3api put-object --bucket $S3_BUCKET --key $CI_BUILD_REF_NAME/$BUILD_PLATFORM/whisper$S3WIN.sha256 --body whisper$S3WIN.sha256
}

make_archive () {
  echo "add artifacts to archive"
  rm -rf parity.zip
  zip -r parity.zip target/$PLATFORM/release/parity$S3WIN target/$PLATFORM/release/parity-evm$S3WIN target/$PLATFORM/release/ethstore$S3WIN target/$PLATFORM/release/ethkey$S3WIN target/$PLATFORM/release/whisper$S3WIN parity$S3WIN.sha256 parity-evm$S3WIN.sha256 ethstore$S3WIN.sha256 ethkey$S3WIN.sha256 whisper$S3WIN.sha256
}

updater_push_release () {
  echo "push release"

  DATA="commit=$CI_BUILD_REF&sha3=$SHA3&filename=parity$S3WIN&secret=$RELEASES_SECRET"
  # Mainnet
  source scripts/safe_curl.sh $DATA "http://update.parity.io:1337/push-build/$CI_BUILD_REF_NAME/$BUILD_PLATFORM"
  # Kovan
  source scripts/safe_curl.sh $DATA "http://update.parity.io:1338/push-build/$CI_BUILD_REF_NAME/$BUILD_PLATFORM"
}

case $BUILD_PLATFORM in
  x86_64-unknown-linux-gnu)
    #set strip bin
    STRIP_BIN="strip"
    #package extention
    build
    strip_binaries
    calculate_checksums
    make_archive
    push_binaries
    updater_push_release
    ;;
  i686-unknown-linux-gnu)
    STRIP_BIN="strip"
    set_env
    build
    strip_binaries
    calculate_checksums
    make_archive
    push_binaries
    ;;
  armv7-unknown-linux-gnueabihf)
    STRIP_BIN="arm-linux-gnueabihf-strip"
    set_env
    build
    strip_binaries
    calculate_checksums
    make_archive
    push_binaries
    ;;
  arm-unknown-linux-gnueabihf)
    STRIP_BIN="arm-linux-gnueabihf-strip"
    set_env
    build
    strip_binaries
    calculate_checksums
    make_archive
    push_binaries
    ;;
  aarch64-unknown-linux-gnu)
    STRIP_BIN="aarch64-linux-gnu-strip"
    set_env
    build
    strip_binaries
    calculate_checksums
    make_archive
    push_binaries
    ;;
  x86_64-apple-darwin)
    STRIP_BIN="strip"
    PLATFORM="x86_64-apple-darwin"
    build
    strip_binaries
    calculate_checksums
    make_archive
    push_binaries
    updater_push_release
    ;;
  x86_64-unknown-snap-gnu)
    ARC="amd64"
    EXT="snap"
    apt update
    apt install -y expect zip rhash
    snapcraft clean
    echo "Prepare snapcraft.yaml for build on Gitlab CI in Docker image"
    sed -i 's/git/'"$VER"'/g' snap/snapcraft.yaml
    if [[ "$CI_BUILD_REF_NAME" = "stable" || "$CI_BUILD_REF_NAME" = "beta" || "$VER" == *1.10* || "$VER" == *1.11* ]];
      then
        sed -i -e 's/grade: devel/grade: stable/' snap/snapcraft.yaml;
    fi
    mv -f snap/snapcraft.yaml snapcraft.yaml
    snapcraft -d
    snapcraft_login=$(expect -c "
      spawn snapcraft login
      expect \"Email:\"
      send \"$SNAP_EMAIL\n\"
      expect \"Password:\"
      send \"$SNAP_PASS\n\"
      expect \"\$\"
      ")
    echo "$snapcraft_login"
    snapcraft push "parity_"$VER"_amd64.snap"
    snapcraft status parity
    snapcraft logout
    $SHA256_BIN "parity_"$VER"_amd64.snap" > "parity_"$VER"_amd64.snap.sha256"
    echo "add artifacts to archive"
    rm -rf parity.zip
    zip -r parity.zip "parity_"$VER"_amd64.snap" "parity_"$VER"_amd64.snap.sha256"
    ;;
  x86_64-pc-windows-msvc)
    set_env_win
    EXT="exe"
    S3WIN=".exe"
    build
    sign_exe
    calculate_checksums
    make_archive
    push_binaries
    updater_push_release
esac
