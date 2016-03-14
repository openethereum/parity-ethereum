FROM ubuntu:14.04

# install tools and dependencies
RUN apt-get update && \
	apt-get install -y \
	# make
	build-essential \
	# add-apt-repository
	software-properties-common \
	curl \
	g++ \
	wget \
	git \
	# evmjit dependencies
	zlib1g-dev \
	libedit-dev

# cmake, llvm and rocksdb ppas. then update ppas
RUN add-apt-repository -y "ppa:george-edison55/cmake-3.x" && \
	add-apt-repository "deb http://llvm.org/apt/trusty/ llvm-toolchain-trusty-3.7 main" && \
	apt-get update && \
	apt-get install -y --force-yes cmake llvm-3.7-dev

# install evmjit
RUN git clone https://github.com/debris/evmjit && \
	cd evmjit && \
	mkdir build && cd build && \
	cmake .. && make && make install && cd

# install multirust
RUN curl -sf https://raw.githubusercontent.com/brson/multirust/master/blastoff.sh | sh -s -- --yes

# export rust LIBRARY_PATH
ENV LIBRARY_PATH /usr/local/lib

# show backtraces
ENV RUST_BACKTRACE 1
