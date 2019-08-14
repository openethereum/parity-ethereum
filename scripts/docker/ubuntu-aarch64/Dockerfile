FROM ubuntu:xenial
WORKDIR /build

# install aarch64(armv8) dependencies and tools
RUN dpkg --add-architecture arm64
RUN echo '# source urls for arm64 \n\
	deb [arch=arm64] http://ports.ubuntu.com/ubuntu-ports/ xenial main \n\
	deb-src [arch=arm64] http://ports.ubuntu.com/ubuntu-ports/ xenial main \n\
	deb [arch=arm64] http://ports.ubuntu.com/ubuntu-ports/ xenial-updates main \n\
	deb-src [arch=arm64] http://ports.ubuntu.com/ubuntu-ports/ xenial-updates main \n\
	deb [arch=arm64] http://ports.ubuntu.com/ubuntu-ports/ xenial-security main \n\
	deb-src [arch=arm64] http://ports.ubuntu.com/ubuntu-ports/ xenial-security main \n # end arm64 section' >> /etc/apt/sources.list &&\
	sed -r 's/deb h/deb \[arch=amd64\] h/g' /etc/apt/sources.list > /tmp/sources-tmp.list && \
	cp /tmp/sources-tmp.list /etc/apt/sources.list&& \
	sed -r 's/deb-src h/deb-src \[arch=amd64\] h/g' /etc/apt/sources.list > /tmp/sources-tmp.list&&cat /etc/apt/sources.list &&\
	cp /tmp/sources-tmp.list /etc/apt/sources.list&& echo "next"&&cat /etc/apt/sources.list

# install tools and dependencies
RUN apt-get -y update && \
	apt-get upgrade -y && \
	apt-get install -y --no-install-recommends \
		curl make cmake file ca-certificates  \
		g++ gcc-aarch64-linux-gnu g++-aarch64-linux-gnu \
		libc6-dev-arm64-cross binutils-aarch64-linux-gnu \
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
	ls /build/parity/target/aarch64-unknown-linux-gnu/release/parity && \
	/usr/bin/aarch64-linux-gnu-strip /build/parity/target/aarch64-unknown-linux-gnu/release/parity

RUN file /build/parity/target/aarch64-unknown-linux-gnu/release/parity

EXPOSE 8080 8545 8180
ENTRYPOINT ["/build/parity/target/aarch64-unknown-linux-gnu/release/parity"]
