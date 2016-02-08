FROM ubuntu:14.04

# install tools and dependencies
RUN apt-get update && \
	apt-get install -y \
  gcc \
	curl \
	git \
	# add-apt-repository
	software-properties-common

# rocksdb ppas. then update ppas
RUN add-apt-repository "deb http://ppa.launchpad.net/giskou/librocksdb/ubuntu trusty main" && \
	apt-get update && \
	apt-get install -y --force-yes librocksdb

# install multirust
RUN curl -sf https://raw.githubusercontent.com/brson/multirust/master/blastoff.sh | sh -s -- --yes

# install nightly and make it default
RUN multirust update nightly && multirust default nightly

# export rust LIBRARY_PATH
ENV LIBRARY_PATH /usr/local/lib

# show backtraces
ENV RUST_BACKTRACE 1

# build parity
RUN git clone https://github.com/ethcore/parity && \
	cd parity && \
	cargo install --features rpc
