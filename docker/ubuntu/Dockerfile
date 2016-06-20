FROM ubuntu:14.04

# install tools and dependencies
RUN apt-get update && \
	apt-get install -y \
	g++ \
	curl \
	git \
	make

# install rustup
RUN curl https://sh.rustup.rs -sSf | sh -s -- -y

# rustup directory
ENV PATH /root/.cargo/bin:$PATH

# show backtraces
ENV RUST_BACKTRACE 1

# build parity
RUN git clone https://github.com/ethcore/parity && \
	cd parity && \
	cargo build --release
