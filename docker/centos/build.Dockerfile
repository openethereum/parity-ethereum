FROM centos:latest

WORKDIR /build

#ENV for build TAG
ARG PARITY_BUILD_TAG
ENV PARITY_BUILD_TAG ${PARITY_BUILD_TAG:-master}
RUN echo "Build tag:" $PARITY_BUILD_TAG

#ENV for build REPO
ARG PARITY_BUILD_REPO
ENV PARITY_BUILD_REPO ${PARITY_BUILD_REPO:-https://github.com/paritytech/parity-ethereum}
RUN echo "Build repo:" $PARITY_BUILD_REPO

RUN yum -y update && \
    yum install -y systemd-devel git make gcc-c++ gcc file binutils && \
    curl -L "https://cmake.org/files/v3.12/cmake-3.12.0-Linux-x86_64.tar.gz" -o cmake.tar.gz && \
    tar -xzf cmake.tar.gz && \
    cp -r cmake-3.12.0-Linux-x86_64/* /usr/ && \
    rm -rf cmake-3.12.0-Linux-x86_64 && \
    rm -rf cmake.tar.gz && \
    curl https://sh.rustup.rs -sSf | sh -s -- -y && \
    PATH=/root/.cargo/bin:$PATH && \
    RUST_BACKTRACE=1 && \

    rustc -vV && \
    cargo -V && \
    gcc -v && \
    g++ -v && \
    cmake --version && \
    

    git clone $PARITY_BUILD_REPO && \
    cd parity-ethereum && \
    git pull && \
    git checkout $PARITY_BUILD_TAG && \
    cargo build --verbose --release --features final && \
    strip /build/parity-ethereum/target/release/parity && \
    file /build/parity-ethereum/target/release/parity


