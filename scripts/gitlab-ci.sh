#!/bin/bash

linux_stable(){
     cargo build -j $(nproc) --release --features final $CARGOFLAGS
     cargo build -j $(nproc) --release -p evmbin
     cargo build -j $(nproc) --release -p ethstore-cli
     cargo build -j $(nproc) --release -p ethkey-cli
     strip target/release/parity
     strip target/release/parity-evm
     strip target/release/ethstore
     strip target/release/ethkey
     export SHA3=$(target/release/parity tools hash target/release/parity)
     md5sum target/release/parity > parity.md5
     sh scripts/deb-build.sh amd64
     cp target/release/parity deb/usr/bin/parity
     cp target/release/parity-evm deb/usr/bin/parity-evm
     cp target/release/ethstore deb/usr/bin/ethstore
     cp target/release/ethkey deb/usr/bin/ethkey
     export VER=$(grep -m 1 version Cargo.toml | awk '{print $3}' | tr -d '"' | tr -d "\n")
     dpkg-deb -b deb "parity_"$VER"_amd64.deb"
     md5sum "parity_"$VER"_amd64.deb" > "parity_"$VER"_amd64.deb.md5"
     aws configure set aws_access_key_id $s3_key
     aws configure set aws_secret_access_key $s3_secret
     if [[ $CI_BUILD_REF_NAME =~ ^(master|beta|stable|nightly)$ ]]; then export S3_BUCKET=builds-parity-published; else export S3_BUCKET=builds-parity; fi
     aws s3 rm --recursive s3://$S3_BUCKET/$CI_BUILD_REF_NAME/x86_64-unknown-linux-gnu
     aws s3api put-object --bucket $S3_BUCKET --key $CI_BUILD_REF_NAME/x86_64-unknown-linux-gnu/parity --body target/release/parity
     aws s3api put-object --bucket $S3_BUCKET --key $CI_BUILD_REF_NAME/x86_64-unknown-linux-gnu/parity.md5 --body parity.md5
     aws s3api put-object --bucket $S3_BUCKET --key $CI_BUILD_REF_NAME/x86_64-unknown-linux-gnu/"parity_"$VER"_amd64.deb" --body "parity_"$VER"_amd64.deb"
     aws s3api put-object --bucket $S3_BUCKET --key $CI_BUILD_REF_NAME/x86_64-unknown-linux-gnu/"parity_"$VER"_amd64.deb.md5" --body "parity_"$VER"_amd64.deb.md5"
     curl --data "commit=$CI_BUILD_REF&sha3=$SHA3&filename=parity&secret=$RELEASES_SECRET" http://update.parity.io:1337/push-build/$CI_BUILD_REF_NAME/x86_64-unknown-linux-gnu
     curl --data "commit=$CI_BUILD_REF&sha3=$SHA3&filename=parity&secret=$RELEASES_SECRET" http://update.parity.io:1338/push-build/$CI_BUILD_REF_NAME/x86_64-unknown-linux-gnu
}

linux_snap(){
     export VER=$(grep -m 1 version Cargo.toml | awk '{print $3}' | tr -d '"' | tr -d "\n")
     cd snap
     rm -rf *snap
     sed -i 's/master/'"$VER"'/g' snapcraft.yaml
     echo "Version:"$VER
     snapcraft
     ls
     cp "parity_"$CI_BUILD"_REF_NAME_amd64.snap" "parity_"$VER"_amd64.snap"
     md5sum "parity_"$VER"_amd64.snap" > "parity_"$VER"_amd64.snap.md5"
     aws configure set aws_access_key_id $s3_key
     aws configure set aws_secret_access_key $s3_secret
     if [[ $CI_BUILD_REF_NAME =~ ^(master|beta|stable|nightly)$ ]]; then export S3_BUCKET=builds-parity-published; else export S3_BUCKET=builds-parity; fi
     aws s3api put-object --bucket $S3_BUCKET --key $CI_BUILD_REF_NAME/x86_64-unknown-linux-gnu/"parity_"$VER"_amd64.snap" --body "parity_"$VER"_amd64.snap"
     aws s3api put-object --bucket $S3_BUCKET --key $CI_BUILD_REF_NAME/x86_64-unknown-linux-gnu/"parity_"$VER"_amd64.snap.md5" --body "parity_"$VER"_amd64.snap.md5"
     curl --data "commit=$CI_BUILD_REF&sha3=$SHA3&filename=parity&secret=$RELEASES_SECRET" http://update.parity.io:1337/push-build/$CI_BUILD_REF_NAME/x86_64-unknown-linux-gnu
     curl --data "commit=$CI_BUILD_REF&sha3=$SHA3&filename=parity&secret=$RELEASES_SECRET" http://update.parity.io:1338/push-build/$CI_BUILD_REF_NAME/x86_64-unknown-linux-gnu
}

linux_stable_debian(){
     cargo build -j $(nproc) --release --features final $CARGOFLAGS
     cargo build -j $(nproc) --release -p evmbin
     cargo build -j $(nproc) --release -p ethstore-cli
     cargo build -j $(nproc) --release -p ethkey-cli
     strip target/release/parity
     strip target/release/parity-evm
     strip target/release/ethstore
     strip target/release/ethkey
     export SHA3=$(target/release/parity tools hash target/release/parity)
     md5sum target/release/parity > parity.md5
     sh scripts/deb-build.sh amd64
     cp target/release/parity deb/usr/bin/parity
     cp target/release/parity-evm deb/usr/bin/parity-evm
     cp target/release/ethstore deb/usr/bin/ethstore
     cp target/release/ethkey deb/usr/bin/ethkey
     export VER=$(grep -m 1 version Cargo.toml | awk '{print $3}' | tr -d '"' | tr -d "\n")
     dpkg-deb -b deb "parity_"$VER"_amd64.deb"
     md5sum "parity_"$VER"_amd64.deb" > "parity_"$VER"_amd64.deb.md5"
     aws configure set aws_access_key_id $s3_key
     aws configure set aws_secret_access_key $s3_secret
     if [[ $CI_BUILD_REF_NAME =~ ^(master|beta|stable|nightly)$ ]]; then export S3_BUCKET=builds-parity-published; else export S3_BUCKET=builds-parity; fi
     aws s3 rm --recursive s3://$S3_BUCKET/$CI_BUILD_REF_NAME/x86_64-unknown-debian-gnu
     aws s3api put-object --bucket $S3_BUCKET --key $CI_BUILD_REF_NAME/x86_64-unknown-debian-gnu/parity --body target/release/parity
     aws s3api put-object --bucket $S3_BUCKET --key $CI_BUILD_REF_NAME/x86_64-unknown-debian-gnu/parity.md5 --body parity.md5
     aws s3api put-object --bucket $S3_BUCKET --key $CI_BUILD_REF_NAME/x86_64-unknown-debian-gnu/"parity_"$VER"_amd64.deb" --body "parity_"$VER"_amd64.deb"
     aws s3api put-object --bucket $S3_BUCKET --key $CI_BUILD_REF_NAME/x86_64-unknown-debian-gnu/"parity_"$VER"_amd64.deb.md5" --body "parity_"$VER"_amd64.deb.md5"
     curl --data "commit=$CI_BUILD_REF&sha3=$SHA3&filename=parity&secret=$RELEASES_SECRET" http://update.parity.io:1337/push-build/$CI_BUILD_REF_NAME/x86_64-unknown-debian-gnu
     curl --data "commit=$CI_BUILD_REF&sha3=$SHA3&filename=parity&secret=$RELEASES_SECRET" http://update.parity.io:1338/push-build/$CI_BUILD_REF_NAME/x86_64-unknown-debian-gnu
}

linux_beta(){
     rustup default beta
     cargo build -j $(nproc) --release $CARGOFLAGS
     strip target/release/parity
}

linux_nightly(){
     rustup default nightly
     cargo build -j $(nproc) --release $CARGOFLAGS
     strip target/release/parity
}

linux_centos(){
     export CXX="g++"
     export CC="gcc"
     export PLATFORM=x86_64-unknown-centos-gnu
     cargo build -j $(nproc) --release --features final $CARGOFLAGS
     cargo build -j $(nproc) --release -p evmbin
     cargo build -j $(nproc) --release -p ethstore-cli
     cargo build -j $(nproc) --release -p ethkey-cli
     strip target/release/parity
     strip target/release/parity-evm
     strip target/release/ethstore
     strip target/release/ethkey
     md5sum target/release/parity > parity.md5
     md5sum target/release/parity-evm > parity-evm.md5
     md5sum target/release/ethstore > ethstore.md5
     md5sum target/release/ethkey > ethkey.md5
     export SHA3=$(target/release/parity tools hash target/release/parity)
     aws configure set aws_access_key_id $s3_key
     aws configure set aws_secret_access_key $s3_secret
     if [[ $CI_BUILD_REF_NAME =~ ^(master|beta|stable|nightly)$ ]]; then export S3_BUCKET=builds-parity-published; else export S3_BUCKET=builds-parity; fi
     aws s3 rm --recursive s3://$S3_BUCKET/$CI_BUILD_REF_NAME/x86_64-unknown-centos-gnu
     aws s3api put-object --bucket builds-parity --key $CI_BUILD_REF_NAME/x86_64-unknown-centos-gnu/parity --body target/release/parity
     aws s3api put-object --bucket builds-parity --key $CI_BUILD_REF_NAME/x86_64-unknown-centos-gnu/parity.md5 --body parity.md5
     aws s3api put-object --bucket builds-parity --key $CI_BUILD_REF_NAME/x86_64-unknown-centos-gnu/parity-evm --body target/release/parity-evm
     aws s3api put-object --bucket builds-parity --key $CI_BUILD_REF_NAME/x86_64-unknown-centos-gnu/parity-evm.md5 --body parity-evm.md5
     aws s3api put-object --bucket builds-parity --key $CI_BUILD_REF_NAME/x86_64-unknown-centos-gnu/ethstore --body target/release/ethstore
     aws s3api put-object --bucket builds-parity --key $CI_BUILD_REF_NAME/x86_64-unknown-centos-gnu/ethstore.md5 --body ethstore.md5
     aws s3api put-object --bucket builds-parity --key $CI_BUILD_REF_NAME/x86_64-unknown-centos-gnu/ethkey --body target/release/ethkey
     aws s3api put-object --bucket builds-parity --key $CI_BUILD_REF_NAME/x86_64-unknown-centos-gnu/ethkey.md5 --body ethkey.md5
     curl --data "commit=$CI_BUILD_REF&sha3=$SHA3&filename=parity&secret=$RELEASES_SECRET" http://update.parity.io:1337/push-build/$CI_BUILD_REF_NAME/$PLATFORM
     curl --data "commit=$CI_BUILD_REF&sha3=$SHA3&filename=parity&secret=$RELEASES_SECRET" http://update.parity.io:1338/push-build/$CI_BUILD_REF_NAME/$PLATFORM
}

linux_i686(){
     export HOST_CC=gcc
     export HOST_CXX=g++
     export COMMIT=$(git rev-parse HEAD)
     export PLATFORM=i686-unknown-linux-gnu
     cargo build -j $(nproc) --target $PLATFORM --features final --release $CARGOFLAGS
     cargo build -j $(nproc) --target $PLATFORM --release -p evmbin
     cargo build -j $(nproc) --target $PLATFORM --release -p ethstore-cli
     cargo build -j $(nproc) --target $PLATFORM --release -p ethkey-cli
     strip target/$PLATFORM/release/parity
     strip target/$PLATFORM/release/parity-evm
     strip target/$PLATFORM/release/ethstore
     strip target/$PLATFORM/release/ethkey
     strip target/$PLATFORM/release/parity
     md5sum target/$PLATFORM/release/parity > parity.md5
     export SHA3=$(target/$PLATFORM/release/parity tools hash target/$PLATFORM/release/parity)
     sh scripts/deb-build.sh i386
     cp target/$PLATFORM/release/parity deb/usr/bin/parity
     cp target/$PLATFORM/release/parity-evm deb/usr/bin/parity-evm
     cp target/$PLATFORM/release/ethstore deb/usr/bin/ethstore
     cp target/$PLATFORM/release/ethkey deb/usr/bin/ethkey
     export VER=$(grep -m 1 version Cargo.toml | awk '{print $3}' | tr -d '"' | tr -d "\n")
     dpkg-deb -b deb "parity_"$VER"_i386.deb"
     md5sum "parity_"$VER"_i386.deb" > "parity_"$VER"_i386.deb.md5"
     aws configure set aws_access_key_id $s3_key
     aws configure set aws_secret_access_key $s3_secret
     if [[ $CI_BUILD_REF_NAME =~ ^(master|beta|stable|nightly)$ ]]; then export S3_BUCKET=builds-parity-published; else export S3_BUCKET=builds-parity; fi
     aws s3 rm --recursive s3://$S3_BUCKET/$CI_BUILD_REF_NAME/$PLATFORM
     aws s3api put-object --bucket $S3_BUCKET --key $CI_BUILD_REF_NAME/$PLATFORM/parity --body target/$PLATFORM/release/parity
     aws s3api put-object --bucket $S3_BUCKET --key $CI_BUILD_REF_NAME/$PLATFORM/parity.md5 --body parity.md5
     aws s3api put-object --bucket $S3_BUCKET --key $CI_BUILD_REF_NAME/$PLATFORM/"parity_"$VER"_i386.deb" --body "parity_"$VER"_i386.deb"
     aws s3api put-object --bucket $S3_BUCKET --key $CI_BUILD_REF_NAME/$PLATFORM/"parity_"$VER"_i386.deb.md5" --body "parity_"$VER"_i386.deb.md5"
     curl --data "commit=$CI_BUILD_REF&sha3=$SHA3&filename=parity&secret=$RELEASES_SECRET" http://update.parity.io:1337/push-build/$CI_BUILD_REF_NAME/$PLATFORM
     curl --data "commit=$CI_BUILD_REF&sha3=$SHA3&filename=parity&secret=$RELEASES_SECRET" http://update.parity.io:1338/push-build/$CI_BUILD_REF_NAME/$PLATFORM
}

linux_armv7(){
     export CC=arm-linux-gnueabihf-gcc
     export CXX=arm-linux-gnueabihf-g++
     export HOST_CC=gcc
     export HOST_CXX=g++
     export PLATFORM=armv7-unknown-linux-gnueabihf
     rm -rf .cargo
     mkdir -p .cargo
     echo "[target.$PLATFORM]" >> .cargo/config
     echo "linker= \"arm-linux-gnueabihf-gcc\"" >> .cargo/config
     cat .cargo/config
     cargo build -j $(nproc) --target $PLATFORM --features final --release $CARGOFLAGS
     cargo build -j $(nproc) --target $PLATFORM --release -p evmbin
     cargo build -j $(nproc) --target $PLATFORM --release -p ethstore-cli
     cargo build -j $(nproc) --target $PLATFORM --release -p ethkey-cli
     md5sum target/$PLATFORM/release/parity > parity.md5
     export SHA3=$(target/$PLATFORM/release/parity tools hash target/$PLATFORM/release/parity)
     sh scripts/deb-build.sh i386
     arm-linux-gnueabihf-strip target/$PLATFORM/release/parity
     arm-linux-gnueabihf-strip target/$PLATFORM/release/parity-evm
     arm-linux-gnueabihf-strip target/$PLATFORM/release/ethstore
     arm-linux-gnueabihf-strip target/$PLATFORM/release/ethkey
     export SHA3=$(rhash --sha3-256 target/$PLATFORM/release/parity -p %h)
     md5sum target/$PLATFORM/release/parity > parity.md5
     sh scripts/deb-build.sh armhf
     cp target/$PLATFORM/release/parity deb/usr/bin/parity
     cp target/$PLATFORM/release/parity-evm deb/usr/bin/parity-evm
     cp target/$PLATFORM/release/ethstore deb/usr/bin/ethstore
     cp target/$PLATFORM/release/ethkey deb/usr/bin/ethkey
     export VER=$(grep -m 1 version Cargo.toml | awk '{print $3}' | tr -d '"' | tr -d "\n")
     dpkg-deb -b deb "parity_"$VER"_armhf.deb"
     md5sum "parity_"$VER"_armhf.deb" > "parity_"$VER"_armhf.deb.md5"
     aws configure set aws_access_key_id $s3_key
     aws configure set aws_secret_access_key $s3_secret
     if [[ $CI_BUILD_REF_NAME =~ ^(master|beta|stable|nightly)$ ]]; then export S3_BUCKET=builds-parity-published; else export S3_BUCKET=builds-parity; fi
     aws s3 rm --recursive s3://$S3_BUCKET/$CI_BUILD_REF_NAME/$PLATFORM
     aws s3api put-object --bucket $S3_BUCKET --key $CI_BUILD_REF_NAME/$PLATFORM/parity --body target/$PLATFORM/release/parity
     aws s3api put-object --bucket $S3_BUCKET --key $CI_BUILD_REF_NAME/$PLATFORM/parity.md5 --body parity.md5
     aws s3api put-object --bucket $S3_BUCKET --key $CI_BUILD_REF_NAME/$PLATFORM/"parity_"$VER"_armhf.deb" --body "parity_"$VER"_armhf.deb"
     aws s3api put-object --bucket $S3_BUCKET --key $CI_BUILD_REF_NAME/$PLATFORM/"parity_"$VER"_armhf.deb.md5" --body "parity_"$VER"_armhf.deb.md5"
     curl --data "commit=$CI_BUILD_REF&sha3=$SHA3&filename=parity&secret=$RELEASES_SECRET" http://update.parity.io:1337/push-build/$CI_BUILD_REF_NAME/$PLATFORM
     curl --data "commit=$CI_BUILD_REF&sha3=$SHA3&filename=parity&secret=$RELEASES_SECRET" http://update.parity.io:1338/push-build/$CI_BUILD_REF_NAME/$PLATFORM
}

linux_arm(){
     export CC=arm-linux-gnueabihf-gcc
     export CXX=arm-linux-gnueabihf-g++
     export HOST_CC=gcc
     export HOST_CXX=g++
     export PLATFORM=arm-unknown-linux-gnueabihf
     rm -rf .cargo
     mkdir -p .cargo
     echo "[target.$PLATFORM]" >> .cargo/config
     echo "linker= \"arm-linux-gnueabihf-gcc\"" >> .cargo/config
     cat .cargo/config
     cargo build -j $(nproc) --target $PLATFORM --features final --release $CARGOFLAGS
     cargo build -j $(nproc) --target $PLATFORM --release -p evmbin
     cargo build -j $(nproc) --target $PLATFORM --release -p ethstore-cli
     cargo build -j $(nproc) --target $PLATFORM --release -p ethkey-cli
     arm-linux-gnueabihf-strip target/$PLATFORM/release/parity
     arm-linux-gnueabihf-strip target/$PLATFORM/release/parity-evm
     arm-linux-gnueabihf-strip target/$PLATFORM/release/ethstore
     arm-linux-gnueabihf-strip target/$PLATFORM/release/ethkey
     export SHA3=$(rhash --sha3-256 target/$PLATFORM/release/parity -p %h)
     md5sum target/$PLATFORM/release/parity > parity.md5
     sh scripts/deb-build.sh armhf
     cp target/$PLATFORM/release/parity deb/usr/bin/parity
     cp target/$PLATFORM/release/parity-evm deb/usr/bin/parity-evm
     cp target/$PLATFORM/release/ethstore deb/usr/bin/ethstore
     cp target/$PLATFORM/release/ethkey deb/usr/bin/ethkey
     export VER=$(grep -m 1 version Cargo.toml | awk '{print $3}' | tr -d '"' | tr -d "\n")
     dpkg-deb -b deb "parity_"$VER"_armhf.deb"
     md5sum "parity_"$VER"_armhf.deb" > "parity_"$VER"_armhf.deb.md5"
     aws configure set aws_access_key_id $s3_key
     aws configure set aws_secret_access_key $s3_secret
     if [[ $CI_BUILD_REF_NAME =~ ^(master|beta|stable|nightly)$ ]]; then export S3_BUCKET=builds-parity-published; else export S3_BUCKET=builds-parity; fi
     aws s3 rm --recursive s3://$S3_BUCKET/$CI_BUILD_REF_NAME/$PLATFORM
     aws s3api put-object --bucket $S3_BUCKET --key $CI_BUILD_REF_NAME/$PLATFORM/parity --body target/$PLATFORM/release/parity
     aws s3api put-object --bucket $S3_BUCKET --key $CI_BUILD_REF_NAME/$PLATFORM/parity.md5 --body parity.md5
     aws s3api put-object --bucket $S3_BUCKET --key $CI_BUILD_REF_NAME/$PLATFORM/"parity_"$VER"_armhf.deb" --body "parity_"$VER"_armhf.deb"
     aws s3api put-object --bucket $S3_BUCKET --key $CI_BUILD_REF_NAME/$PLATFORM/"parity_"$VER"_armhf.deb.md5" --body "parity_"$VER"_armhf.deb.md5"
     curl --data "commit=$CI_BUILD_REF&sha3=$SHA3&filename=parity&secret=$RELEASES_SECRET" http://update.parity.io:1337/push-build/$CI_BUILD_REF_NAME/$PLATFORM
     curl --data "commit=$CI_BUILD_REF&sha3=$SHA3&filename=parity&secret=$RELEASES_SECRET" http://update.parity.io:1338/push-build/$CI_BUILD_REF_NAME/$PLATFORM
}

linux_aarch64(){
     export CC=aarch64-linux-gnu-gcc
     export CXX=aarch64-linux-gnu-g++
     export HOST_CC=gcc
     export HOST_CXX=g++
     export PLATFORM=aarch64-unknown-linux-gnu
     rm -rf .cargo
     mkdir -p .cargo
     echo "[target.$PLATFORM]" >> .cargo/config
     echo "linker= \"aarch64-linux-gnu-gcc\"" >> .cargo/config
     cat .cargo/config
     cargo build -j $(nproc) --target $PLATFORM --features final --release $CARGOFLAGS
     cargo build -j $(nproc) --target $PLATFORM --release -p evmbin
     cargo build -j $(nproc) --target $PLATFORM --release -p ethstore-cli
     cargo build -j $(nproc) --target $PLATFORM --release -p ethkey-cli
     aarch64-linux-gnu-strip target/$PLATFORM/release/parity
     aarch64-linux-gnu-strip target/$PLATFORM/release/parity-evm
     aarch64-linux-gnu-strip target/$PLATFORM/release/ethstore
     aarch64-linux-gnu-strip target/$PLATFORM/release/ethkey
     export SHA3=$(rhash --sha3-256 target/$PLATFORM/release/parity -p %h)
     md5sum target/$PLATFORM/release/parity > parity.md5
     sh scripts/deb-build.sh arm64
     cp target/$PLATFORM/release/parity deb/usr/bin/parity
     cp target/$PLATFORM/release/parity-evm deb/usr/bin/parity-evm
     cp target/$PLATFORM/release/ethstore deb/usr/bin/ethstore
     cp target/$PLATFORM/release/ethkey deb/usr/bin/ethkey
     export VER=$(grep -m 1 version Cargo.toml | awk '{print $3}' | tr -d '"' | tr -d "\n")
     dpkg-deb -b deb "parity_"$VER"_arm64.deb"
     md5sum "parity_"$VER"_arm64.deb" > "parity_"$VER"_arm64.deb.md5"
     aws configure set aws_access_key_id $s3_key
     aws configure set aws_secret_access_key $s3_secret
     if [[ $CI_BUILD_REF_NAME =~ ^(master|beta|stable|nightly)$ ]]; then export S3_BUCKET=builds-parity-published; else export S3_BUCKET=builds-parity; fi
     aws s3 rm --recursive s3://$S3_BUCKET/$CI_BUILD_REF_NAME/$PLATFORM
     aws s3api put-object --bucket $S3_BUCKET --key $CI_BUILD_REF_NAME/$PLATFORM/parity.md5 --body parity.md5
     aws s3api put-object --bucket $S3_BUCKET --key $CI_BUILD_REF_NAME/$PLATFORM/"parity_"$VER"_arm64.deb" --body "parity_"$VER"_arm64.deb"
     aws s3api put-object --bucket $S3_BUCKET --key $CI_BUILD_REF_NAME/$PLATFORM/"parity_"$VER"_arm64.deb.md5" --body "parity_"$VER"_arm64.deb.md5"
     curl --data "commit=$CI_BUILD_REF&sha3=$SHA3&filename=parity&secret=$RELEASES_SECRET" http://update.parity.io:1337/push-build/$CI_BUILD_REF_NAME/$PLATFORM
     curl --data "commit=$CI_BUILD_REF&sha3=$SHA3&filename=parity&secret=$RELEASES_SECRET" http://update.parity.io:1338/push-build/$CI_BUILD_REF_NAME/$PLATFORM
}

darwin(){
      export COMMIT=$(git rev-parse HEAD)
      export PLATFORM=x86_64-apple-darwin
      rustup default stable
      cargo clean
      cargo build -j 8 --features final --release #$CARGOFLAGS
      cargo build -j 8 --release -p ethstore-cli #$CARGOFLAGS
      cargo build -j 8 --release -p ethkey-cli #$CARGOFLAGS
      cargo build -j 8 --release -p evmbin #$CARGOFLAGS
      rm -rf parity.md5
      md5sum target/release/parity > parity.md5
      export SHA3=$(target/release/parity tools hash target/release/parity)
      cd mac
      xcodebuild -configuration Release
      cd ..
      packagesbuild -v mac/Parity.pkgproj
      productsign --sign 'Developer ID Installer: PARITY TECHNOLOGIES LIMITED (P2PX3JU8FT)' target/release/Parity\ Ethereum.pkg target/release/Parity\ Ethereum-signed.pkg
      export VER=$(grep -m 1 version Cargo.toml | awk '{print $3}' | tr -d '"' | tr -d "\n")
      mv target/release/Parity\ Ethereum-signed.pkg "parity-"$VER"-macos-installer.pkg"
      md5sum "parity-"$VER"-macos-installer.pkg" >> "parity-"$VER"-macos-installer.pkg.md5"
      aws configure set aws_access_key_id $s3_key
      aws configure set aws_secret_access_key $s3_secret
      if [[ $CI_BUILD_REF_NAME =~ ^(master|beta|stable|nightly)$ ]]; then export S3_BUCKET=builds-parity-published; else export S3_BUCKET=builds-parity; fi
      aws s3 rm --recursive s3://$S3_BUCKET/$CI_BUILD_REF_NAME/$PLATFORM
      aws s3api put-object --bucket $S3_BUCKET --key $CI_BUILD_REF_NAME/$PLATFORM/parity --body target/release/parity
      aws s3api put-object --bucket $S3_BUCKET --key $CI_BUILD_REF_NAME/$PLATFORM/parity.md5 --body parity.md5
      aws s3api put-object --bucket $S3_BUCKET --key $CI_BUILD_REF_NAME/$PLATFORM/"parity-"$VER"-macos-installer.pkg" --body "parity-"$VER"-macos-installer.pkg"
      aws s3api put-object --bucket $S3_BUCKET --key $CI_BUILD_REF_NAME/$PLATFORM/"parity-"$VER"-macos-installer.pkg.md5" --body "parity-"$VER"-macos-installer.pkg.md5"
      curl --data "commit=$CI_BUILD_REF&sha3=$SHA3&filename=parity&secret=$RELEASES_SECRET" http://update.parity.io:1337/push-build/$CI_BUILD_REF_NAME/$PLATFORM
      curl --data "commit=$CI_BUILD_REF&sha3=$SHA3&filename=parity&secret=$RELEASES_SECRET" http://update.parity.io:1338/push-build/$CI_BUILD_REF_NAME/$PLATFORM
}

windows(){
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
     curl --data "commit=%CI_BUILD_REF%&sha3=%SHA3%&filename=parity.exe&secret=%RELEASES_SECRET%" http://update.parity.io:1337/push-build/%CI_BUILD_REF_NAME%/%PLATFORM%
     curl --data "commit=%CI_BUILD_REF%&sha3=%SHA3%&filename=parity.exe&secret=%RELEASES_SECRET%" http://update.parity.io:1338/push-build/%CI_BUILD_REF_NAME%/%PLATFORM%
}

docker_build(){
     if [[ "$CI_BUILD_REF_NAME" == "beta-release" ]]; then DOCKER_TAG="latest"; else DOCKER_TAG=$CI_BUILD_REF_NAME; fi
     echo "Tag:" $DOCKER_TAG
     docker login -u $Docker_Hub_User_Parity -p $Docker_Hub_Pass_Parity
     sh scripts/docker-build.sh $DOCKER_TAG
     docker logout
}

test_rust_stable_before_script(){
     git submodule update --init --recursive
     export RUST_FILES_MODIFIED=$(git --no-pager diff --name-only $CI_BUILD_REF^ $CI_BUILD_REF | grep -v -e ^js -e ^\\. -e ^LICENSE -e ^README.md -e ^test.sh -e ^windows/ -e ^scripts/ -e^mac/ -e ^nsis/ | wc -l)
}

test_rust_stable(){
     rustup show
     export RUST_BACKTRACE=1
     if [[ $RUST_FILES_MODIFIED -eq 0 ]]; then echo "Skipping Rust tests since no Rust files modified."; else ./test.sh $CARGOFLAGS; fi
     if [[ "$CI_BUILD_REF_NAME" == "nightly" ]]; then sh scripts/aura-test.sh; fi
}

js_test_before_script(){
     git submodule update --init --recursive
     export JS_FILES_MODIFIED=$(git --no-pager diff --name-only $CI_BUILD_REF^ $CI_BUILD_REF | grep ^js/ | wc -l)
     if [[ $JS_FILES_MODIFIED -eq 0 ]]; then echo "Skipping JS deps install since no JS files modified."; else ./js/scripts/install-deps.sh;fi
     export JS_OLD_FILES_MODIFIED=$(git --no-pager diff --name-only $CI_BUILD_REF^ $CI_BUILD_REF | grep ^js-old/ | wc -l)
     if [[ $JS_OLD_FILES_MODIFIED -eq 0  ]]; then echo "Skipping JS (old) deps install since no JS files modified."; else ./js-old/scripts/install-deps.sh;fi
}

js_test(){
     if [[ $JS_FILES_MODIFIED -eq 0 ]]; then echo "Skipping JS lint since no JS files modified."; else ./js/scripts/lint.sh && ./js/scripts/test.sh && ./js/scripts/build.sh; fi
     if [[ $JS_OLD_FILES_MODIFIED -eq 0 ]]; then echo "Skipping JS (old) lint since no JS files modified."; else ./js-old/scripts/lint.sh && ./js-old/scripts/test.sh && ./js-old/scripts/build.sh; fi
}

test_rust_beta_before_script(){
     git submodule update --init --recursive
     export RUST_FILES_MODIFIED=$(git --no-pager diff --name-only $CI_BUILD_REF^ $CI_BUILD_REF | grep -v -e ^js -e ^\\. -e ^LICENSE -e ^README.md -e ^appveyor.yml -e ^test.sh -e ^windows/ -e ^scripts/ -e^mac/ -e ^nsis/ | wc -l)
}

test_rust_beta(){
     rustup default beta
     export RUST_BACKTRACE=1
     if [[ $RUST_FILES_MODIFIED -eq 0 ]]; then echo "Skipping Rust tests since no Rust files modified."; else ./test.sh $CARGOFLAGS; fi
}

test_rust_nightly_before_script(){
     git submodule update --init --recursive
     export RUST_FILES_MODIFIED=$(git --no-pager diff --name-only $CI_BUILD_REF^ $CI_BUILD_REF | grep -v -e ^js -e ^\\. -e ^LICENSE -e ^README.md -e ^appveyor.yml -e ^test.sh -e ^windows/ -e ^scripts/ -e^mac/ -e ^nsis/ | wc -l)
}

test_rust_nightly(){
     rustup default nightly
     export RUST_BACKTRACE=1
     if [[ $RUST_FILES_MODIFIED -eq 0 ]]; then echo "Skipping Rust tests since no Rust files modified."; else ./test.sh $CARGOFLAGS; fi
}

js_release_before_script(){
     export JS_FILES_MODIFIED=$(git --no-pager diff --name-only $CI_BUILD_REF^ $CI_BUILD_REF | grep ^js/ | wc -l)
     echo $JS_FILES_MODIFIED
     if [[ $JS_FILES_MODIFIED -eq 0  ]]; then echo "Skipping JS deps install since no JS files modified."; else ./js/scripts/install-deps.sh;fi
     export JS_OLD_FILES_MODIFIED=$(git --no-pager diff --name-only $CI_BUILD_REF^ $CI_BUILD_REF | grep ^js-old/ | wc -l)
     echo $JS_OLD_FILES_MODIFIED
     if [[ $JS_OLD_FILES_MODIFIED -eq 0  ]]; then echo "Skipping JS (old) deps install since no JS files modified."; else ./js-old/scripts/install-deps.sh;fi
}

js_release(){
     rustup default stable
     echo $JS_FILES_MODIFIED
     if [[ $JS_FILES_MODIFIED -eq 0 ]]; then echo "Skipping JS rebuild since no JS files modified."; else ./js/scripts/build.sh && ./js/scripts/push-precompiled.sh; fi
     echo $JS_OLD_FILES_MODIFIED
     if [[ $JS_OLD_FILES_MODIFIED -eq 0 ]]; then echo "Skipping JS (old) rebuild since no JS files modified."; else ./js-old/scripts/build.sh && ./js-old/scripts/push-precompiled.sh; fi
     if [[ $JS_FILES_MODIFIED -eq 0 ]] && [[ $JS_OLD_FILES_MODIFIED -eq 0 ]]; then echo "Skipping Cargo update since no JS files modified."; else ./js/scripts/push-cargo.sh; fi
}

push_release(){
     rustup default stable
     curl --data "secret=$RELEASES_SECRET" http://update.parity.io:1337/push-release/$CI_BUILD_REF_NAME/$CI_BUILD_REF
     curl --data "secret=$RELEASES_SECRET" http://update.parity.io:1338/push-release/$CI_BUILD_REF_NAME/$CI_BUILD_REF
}

numargs=$#
for ((i=1 ; i <= numargs ; i++))
do
  $1
  shift
done
