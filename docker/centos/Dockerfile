FROM centos:latest
WORKDIR /build
# install tools and dependencies
RUN yum -y update&& \
    yum install -y git make gcc-c++ gcc file binutils
# install rustup
RUN curl -sSf https://static.rust-lang.org/rustup.sh -o rustup.sh &&\
ls&&\
 sh rustup.sh -s -- --disable-sudo
# show backtraces
ENV RUST_BACKTRACE 1
# set compiler
ENV CXX g++
ENV CC gcc
# show tools
RUN rustc -vV && \
cargo -V && \
gcc -v &&\
g++ -v
# build parity
RUN git clone https://github.com/ethcore/parity && \
	cd parity&&\
	git checkout beta && \
        git pull && \
        ls -a&&\
        cargo build --release --verbose && \
	ls /build/parity/target/release/parity &&	\
	strip /build/parity/target/release/parity

RUN file /build/parity/target/release/parity

EXPOSE 8080 8545 8180
ENTRYPOINT ["/build/parity/target/release/parity"]
