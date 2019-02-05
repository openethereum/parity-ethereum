#Building image
FROM centos:latest as builder

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
    cd /build/parity-ethereum && \
    cargo build --verbose --release --features "final secretstore" && \
    strip /build/parity-ethereum/target/release/parity && \
    file /build/parity-ethereum/target/release/parity


# Runtime image
FROM centos:latest

RUN mkdir -p /opt/parity/data && \
    chmod g+rwX /opt/parity/data && \
    curl -L https://github.com/stedolan/jq/releases/download/jq-1.6/jq-linux64 -o /usr/local/bin/jq && \
    chmod +x /usr/local/bin/jq

COPY --from=builder /build/parity-ethereum/target/release /opt/parity/

WORKDIR /opt/parity/data
ADD ./scripts/docker/centos-ocean/entrypoint.sh /entrypoint.sh

# exposing default ports
#
#           secret
#      ipfs store     ui   rpc  ws   listener  discovery
#      ↓    ↓         ↓    ↓    ↓    ↓         ↓
EXPOSE 5001 8082 8083 8180 8545 8546 30303/tcp 30303/udp

# switch to non-root user
# USER 1001

#if no base path provided, assume it's current workdir
CMD ["--base-path","."]
ENTRYPOINT ["/opt/parity/parity"]
