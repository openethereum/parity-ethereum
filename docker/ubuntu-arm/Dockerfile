FROM ubuntu:14.04
WORKDIR /build
# install tools and dependencies
RUN apt-get -y update && \
	apt-get install -y --force-yes --no-install-recommends \
	curl git make g++ gcc-arm-linux-gnueabihf g++-arm-linux-gnueabihf \
	libc6-dev-armhf-cross wget file ca-certificates \
	binutils-arm-linux-gnueabihf \
	&& \
    apt-get clean

# install multirust
RUN curl -sf https://raw.githubusercontent.com/brson/multirust/master/blastoff.sh | sh -s -- --yes
ENV RUST_TARGETS="arm-unknown-linux-gnueabihf"

# multirust add arm--linux-gnuabhf toolchain
RUN multirust add-target stable arm-unknown-linux-gnueabihf

# show backtraces
ENV RUST_BACKTRACE 1
# set compilers
ENV CXX arm-linux-gnueabihf-g++ 
ENV CC arm-linux-gnueabihf-gcc 
# build parity
RUN git clone https://github.com/ethcore/parity && \
	cd parity && \
	git checkout master && \
	wget https://github.com/nix-rust/nix/archive/v0.5.0.tar.gz && \
	tar -xf v0.5.0.tar.gz && \
	rm -rf v0.5.0.tar.gz && \
	wget https://github.com/thkaw/mio/archive/v0.5.x.tar.gz && \
	tar -xf v0.5.x.tar.gz && \
	rm -rf v0.5.x.tar.gz && \
	mkdir -p .cargo && \
  	echo 'paths = ["nix-0.5.0","mio-0.5.x"]\n\
	[target.arm-unknown-linux-gnueabihf]\n\
	linker = "arm-linux-gnueabihf-gcc"\n'\
	>>.cargo/config && \
	cat .cargo/config && \
	rustc -vV && \
	cargo -V && \
	cargo build --target arm-unknown-linux-gnueabihf --release --verbose && \
	ls /build/parity/target/arm-unknown-linux-gnueabihf/release/parity &&	\
	file /build/parity/target/arm-unknown-linux-gnueabihf/release/parity && \
	/usr/bin/arm-linux-gnueabihf-strip /build/parity/target/arm-unknown-linux-gnueabihf/release/parity
RUN file /build/parity/target/arm-unknown-linux-gnueabihf/release/parity
