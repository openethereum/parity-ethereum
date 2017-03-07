FROM ubuntu:14.04
MAINTAINER Parity Technologies <devops@parity.io>
WORKDIR /build
#ENV for build TAG
ARG BUILD_TAG
ENV BUILD_TAG ${BUILD_TAG:-master}
RUN echo $BUILD_TAG
# install tools and dependencies
RUN apt-get update && \
        apt-get install -y --force-yes --no-install-recommends \
        # make
        build-essential \
        # add-apt-repository
        software-properties-common \
        make \
        curl \
        wget \
        git \
        g++ \
        gcc \
        libc6 \
        libc6-dev \
        binutils \
        file \
        openssl \
        libssl-dev \
        libudev-dev \
        pkg-config \
        dpkg-dev \
        # evmjit dependencies
        zlib1g-dev \
        libedit-dev \
	libudev-dev &&\
# cmake and llvm ppa's. then update ppa's
 add-apt-repository -y "ppa:george-edison55/cmake-3.x" && \
        add-apt-repository "deb http://llvm.org/apt/trusty/ llvm-toolchain-trusty-3.7 main" && \
        apt-get update && \
        apt-get install -y --force-yes cmake llvm-3.7-dev && \
# install evmjit
 git clone https://github.com/debris/evmjit && \
        cd evmjit && \
        mkdir build && cd build && \
        cmake .. && make && make install && cd && \
# install rustup
 curl https://sh.rustup.rs -sSf | sh -s -- -y && \
# rustup directory
 PATH=/root/.cargo/bin:$PATH && \
# show backtraces
 RUST_BACKTRACE=1 && \
# build parity
 cd /build&&git clone https://github.com/ethcore/parity && \
        cd parity && \
	git pull&& \
	git checkout $BUILD_TAG && \
        cargo build --verbose --release --features final && \
        #ls /build/parity/target/release/parity && \
        strip /build/parity/target/release/parity && \
 file /build/parity/target/release/parity&&mkdir -p /parity&& cp /build/parity/target/release/parity /parity&&\
#cleanup Docker image
 rm -rf /root/.cargo&&rm -rf /root/.multirust&&rm -rf /root/.rustup&&rm -rf /build&&\
 apt-get purge -y  \
        # make
        build-essential \
        # add-apt-repository
        software-properties-common \
        make \
        curl \
        wget \
        git \
        g++ \
        gcc \
        binutils \
        file \
        pkg-config \
        dpkg-dev \
        # evmjit dependencies
        zlib1g-dev \
        libedit-dev \
        cmake llvm-3.7-dev&&\
 rm -rf /var/lib/apt/lists/*
# setup ENTRYPOINT
EXPOSE 8080 8545 8180
ENTRYPOINT ["/parity/parity"]
