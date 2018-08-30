FROM centos:latest

WORKDIR /build

ADD . /build/parity-ethereum

RUN yum -y update && \
    yum install -y systemd-devel git make gcc-c++ gcc file binutils && \
    curl -L "https://cmake.org/files/v3.12/cmake-3.12.0-Linux-x86_64.tar.gz" -o cmake.tar.gz && \
    tar -xzf cmake.tar.gz && \
    cp -r cmake-3.12.0-Linux-x86_64/* /usr/ && \
    curl https://sh.rustup.rs -sSf | sh -s -- -y && \
    PATH=/root/.cargo/bin:$PATH && \
    RUST_BACKTRACE=1 && \
    rustc -vV && \
    cargo -V && \
    gcc -v && \
    g++ -v && \
    cmake --version && \
    cd parity-ethereum && \
    cargo build --verbose --release --features final && \
    strip /build/parity-ethereum/target/release/parity && \
    file /build/parity-ethereum/target/release/parity


