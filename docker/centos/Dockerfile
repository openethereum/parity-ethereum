FROM centos:latest

WORKDIR /build

#ENV for build TAG
ARG BUILD_TAG
ENV BUILD_TAG ${BUILD_TAG:-master}
RUN echo "Build tag:" $BUILD_TAG

# show backtraces & set compiler
ENV RUST_BACKTRACE 1
ENV CXX g++
ENV CC gcc

# install tools and dependencies, build parity, set folder permissions, clean up
RUN yum -y update && \
    yum install -y systemd-devel git make gcc-c++ gcc file binutils && \
    curl -L "https://cmake.org/files/v3.12/cmake-3.12.0-Linux-x86_64.tar.gz" -o cmake.tar.gz && \
    tar -xzf cmake.tar.gz && \
    cp -r cmake-3.12.0-Linux-x86_64/* /usr/ && \
    rm -rf cmake-3.12.0-Linux-x86_64 && \
    rm -rf cmake.tar.gz && \
    curl -sSf https://static.rust-lang.org/rustup.sh -o rustup.sh && \
    sh rustup.sh --disable-sudo && \

    rustc -vV && \
	cargo -V && \
	gcc -v && \
	g++ -v && \
    cmake --version && \
    
    cd /build && \
    git clone https://github.com/paritytech/parity-ethereum && \
    cd parity-ethereum && \
    git pull && \
    git checkout $BUILD_TAG && \

    cargo build --verbose --release --features final && \
	strip /build/parity-ethereum/target/release/parity && \
    file /build/parity-ethereum/target/release/parity && \
    mkdir -p /opt/parity/release && \
	cp /build/parity-ethereum/target/release/parity /opt/parity/release && \

    mkdir -p /opt/parity/data && \
    chmod g+rwX /opt/parity/data && \

    rm -rf /root/.cargo && \
    rm -rf /root/.multirust && \
    rm -rf /root/.rustup && \
    rm -rf /build && \
    yum clean all -y && \
    rm -rf /var/cache/yum
    
WORKDIR /opt/parity/data

EXPOSE 8080 8545 8180

# switch to non-root user
USER 1001

ENTRYPOINT ["/opt/parity/release/parity"]
