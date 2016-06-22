FROM centos:latest
WORKDIR /build
# install tools and dependencies
RUN yum -y update&& \
    yum install -y git make gcc-c++ gcc file 
# install rustup
RUN curl -sSf https://static.rust-lang.org/rustup.sh -o rustup.sh &&\
ls&&\ 
 sh rustup.sh -s -- --disable-sudo
# show backtraces
ENV RUST_BACKTRACE 1
ENV CXX g++
ENV CC gcc
RUN rustc -vV && \
cargo -V && \
gcc -v &&\
g++ -v
# git clone parity
RUN git clone https://github.com/ethcore/parity && \
	cd parity&&\
        ls -a&&\
        cargo build --release --verbose && \
	ls /build/parity/target/release/parity &&	\
	file /build/parity/target/release/parity && \
RUN file /build/parity/target/release/parity
