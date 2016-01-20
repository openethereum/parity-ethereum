FROM ubuntu:14.04

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

# install rocksdb
RUN wget https://github.com/facebook/rocksdb/archive/rocksdb-3.13.1.tar.gz && \
	tar -zxvf rocksdb-3.13.1.tar.gz && \
	cd rocksdb-rocksdb-3.13.1 && \
	make shared_lib && make install && cd

# install multirust
RUN curl -sf https://raw.githubusercontent.com/brson/multirust/master/blastoff.sh | sh -s -- --yes

# install nightly and make it default
RUN multirust update nightly && multirust default nightly

# export rust LIBRARY_PATH
ENV LIBRARY_PATH /usr/local/lib

# show backtraces
ENV RUST_BACKTRACE 1

# run tests synchronously. Temporary workaround for evmjit cache race condition
ENV RUST_TEST_THREADS 1
