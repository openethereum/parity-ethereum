FROM centos:latest

WORKDIR /build

#ENV for build TAG
ARG BUILD_TAG
ENV BUILD_TAG ${BUILD_TAG:-master}
RUN echo "Build tag:" $BUILD_TAG

#ENV for build REPO
ARG BUILD_REPO
ENV BUILD_REPO ${BUILD_REPO:-https://github.com/paritytech/parity-ethereum}
RUN echo "Build repo:" $BUILD_REPO

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
    
    mkdir -p /opt/parity/data && \
    chmod g+rwX /opt/parity/data && \
    mkdir -p /opt/parity/release && \

    git clone $BUILD_REPO && \
    cd parity-ethereum && \
    git pull && \
    git checkout $BUILD_TAG && \
    cargo build --verbose --release --features final && \
    strip /build/parity-ethereum/target/release/parity && \
    file /build/parity-ethereum/target/release/parity && \
    cp /build/parity-ethereum/target/release/parity /opt/parity/release && \

    rm -rf /root/.cargo && \
    rm -rf /root/.multirust && \
    rm -rf /root/.rustup \
    yum clean all -y && \
    rm -rf /var/cache/yum && \
    cd / && \
    rm -rf /build

WORKDIR /opt/parity/data

EXPOSE 8080 8545 8180

# switch to non-root user
USER 1001

ENTRYPOINT ["/opt/parity/release/parity"]
