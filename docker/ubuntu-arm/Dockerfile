FROM ubuntu:14.04
WORKDIR /build

# install tools and dependencies
RUN apt-get -y update && \
        apt-get install -y --force-yes --no-install-recommends \
        curl git make g++ gcc-arm-linux-gnueabihf g++-arm-linux-gnueabihf \
        libc6-dev-armhf-cross wget file ca-certificates \
        binutils-arm-linux-gnueabihf cmake \
        && \
    apt-get clean

# install rustup
RUN curl https://sh.rustup.rs -sSf | sh -s -- -y

# rustup directory
ENV PATH /root/.cargo/bin:$PATH

ENV RUST_TARGETS="arm-unknown-linux-gnueabihf"

# multirust add arm--linux-gnuabhf toolchain
RUN rustup target add armv7-unknown-linux-gnueabihf

# show backtraces
ENV RUST_BACKTRACE 1

# show tools
RUN rustc -vV && cargo -V

# build parity
ADD . /build/parity
RUN cd parity && \
        mkdir -p .cargo && \
        echo '[target.armv7-unknown-linux-gnueabihf]\n\
        linker = "arm-linux-gnueabihf-gcc"\n'\
        >>.cargo/config && \
        cat .cargo/config && \
        cargo build --target armv7-unknown-linux-gnueabihf --release --verbose && \
        ls /build/parity/target/armv7-unknown-linux-gnueabihf/release/parity && \
        /usr/bin/arm-linux-gnueabihf-strip /build/parity/target/armv7-unknown-linux-gnueabihf/release/parity

RUN file /build/parity/target/armv7-unknown-linux-gnueabihf/release/parity

EXPOSE 8080 8545 8180
ENTRYPOINT ["/build/parity/target/armv7-unknown-linux-gnueabihf/release/parity"]
