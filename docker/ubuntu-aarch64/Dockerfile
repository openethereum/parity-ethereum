FROM ubuntu:14.04
WORKDIR /build

# install tools and dependencies
RUN apt-get -y update && \
        apt-get install -y --force-yes --no-install-recommends \
        curl git make g++ gcc-aarch64-linux-gnu g++-aarch64-linux-gnu \
        libc6-arm64-cross libc6-dev-arm64-cross wget file ca-certificates \
        binutils-aarch64-linux-gnu cmake3 libudev-dev \
        && \
    apt-get clean

# install rustup
RUN curl https://sh.rustup.rs -sSf | sh -s -- -y

# rustup directory
ENV PATH /root/.cargo/bin:$PATH

ENV RUST_TARGETS="aarch64-unknown-linux-gnu"

# multirust add arm--linux-gnuabhf toolchain
RUN rustup target add aarch64-unknown-linux-gnu

# show backtraces
ENV RUST_BACKTRACE 1

# show tools
RUN rustc -vV && cargo -V

# build parity
ADD . /build/parity
RUN cd parity && \
        mkdir -p .cargo && \
        echo '[target.aarch64-unknown-linux-gnu]\n\
        linker = "aarch64-linux-gnu-gcc"\n'\
        >>.cargo/config && \
        cat .cargo/config && \
        cargo build --target aarch64-unknown-linux-gnu --release --verbose && \
        ls /build/parity/target/aarch64-unknown-linux-gnu/release/parity &&     \
        /usr/bin/aarch64-linux-gnu-strip /build/parity/target/aarch64-unknown-linux-gnu/release/parity

RUN file /build/parity/target/aarch64-unknown-linux-gnu/release/parity

EXPOSE 8080 8545 8180
ENTRYPOINT ["/build/parity/target/aarch64-unknown-linux-gnu/release/parity"]
