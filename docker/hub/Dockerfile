FROM ubuntu:xenial
MAINTAINER Parity Technologies <devops@parity.io>
WORKDIR /build
#ENV for build TAG
ARG BUILD_TAG
ENV BUILD_TAG ${BUILD_TAG:-master}
RUN echo "Build tag:" $BUILD_TAG
# install tools and dependencies
RUN apt-get update && \
        apt-get install -y --force-yes --no-install-recommends \
        # make
        build-essential \
        # add-apt-repository
        software-properties-common \
        make \
        cmake \
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
        libudev-dev &&\
# install rustup
 curl https://sh.rustup.rs -sSf | sh -s -- -y && \
# rustup directory
 PATH=/root/.cargo/bin:$PATH && \
# show backtraces
 RUST_BACKTRACE=1 && \
# build parity
cd /build&&git clone https://github.com/paritytech/parity-ethereum && \
        cd parity && \
	git pull&& \
	git checkout $BUILD_TAG && \
        cargo build --verbose --release --features final && \
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
        cmake \
        curl \
        wget \
        git \
        g++ \
        gcc \
        binutils \
        file \
        pkg-config \
        dpkg-dev &&\
 rm -rf /var/lib/apt/lists/*
# setup ENTRYPOINT
EXPOSE 8080 8545 8180
ENTRYPOINT ["/parity/parity"]
