FROM ubuntu:14.04
WORKDIR /build
# install tools and dependencies
RUN apt-get -y update && \
    apt-get install -y --force-yes --no-install-recommends g++ gcc libc6 libc6-dev \
        python binutils curl git make file ca-certificates zip dpkg-dev rhash openssl build-essential pkg-config libssl-dev

# install AWS CLI
RUN curl -O https://bootstrap.pypa.io/get-pip.py && \
    python get-pip.py && \
    pip install awscli

# install nodejs
RUN curl -sL https://deb.nodesource.com/setup_6.x | bash - && \
    apt-get install -y nodejs && \
    apt-get clean

# install rustup
RUN curl https://sh.rustup.rs -sSf | sh -s -- -y

# rustup directory
ENV PATH /root/.cargo/bin:$PATH

# show backtraces
ENV RUST_BACKTRACE 1

# show tools
RUN rustc -vV && \
cargo -V && \
gcc -v &&\
g++ -v

# build parity
RUN git clone https://github.com/ethcore/parity && \
        cd parity && \
        git checkout beta && \
        git pull && \
        cargo build --features final --release --verbose && \
        ls /build/parity/target/release/parity && \
        strip /build/parity/target/release/parity

RUN file /build/parity/target/release/parity

EXPOSE 8080 8545 8180
ENTRYPOINT ["/build/parity/target/release/parity"]
