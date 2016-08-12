FROM ubuntu:14.04
WORKDIR /build
# install tools and dependencies
RUN apt-get update && \
        apt-get install -y \
        # make
        build-essential \
        # add-apt-repository
        software-properties-common \
        curl \
        wget \
        git \
        g++ \
        binutils \
        file \
        # evmjit dependencies
        zlib1g-dev \
        libedit-dev

# cmake and llvm ppas. then update ppas
RUN add-apt-repository -y "ppa:george-edison55/cmake-3.x" && \
        add-apt-repository "deb http://llvm.org/apt/trusty/ llvm-toolchain-trusty-3.7 main" && \
        apt-get update && \
        apt-get install -y --force-yes cmake llvm-3.7-dev

# install evmjit
RUN git clone https://github.com/debris/evmjit && \
        cd evmjit && \
        mkdir build && cd build && \
        cmake .. && make && make install && cd

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
        cargo build --release --features ethcore/jit --verbose && \
        ls /build/parity/target/release/parity &&       \
        strip /build/parity/target/release/parity

RUN file /build/parity/target/release/parity

EXPOSE 8080 8545 8180
ENTRYPOINT ["/build/parity/target/release/parity"]
