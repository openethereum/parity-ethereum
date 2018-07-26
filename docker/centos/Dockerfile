FROM centos:latest
WORKDIR /build

# install tools and dependencies

RUN yum -y update 
RUN yum install -y systemd-devel
RUN yum install -y git make gcc-c++ gcc file binutils 


RUN curl -L "https://cmake.org/files/v3.12/cmake-3.12.0-Linux-x86_64.tar.gz" -o cmake.tar.gz && \
 tar -xzf cmake.tar.gz && \
 cp -r cmake-3.12.0-Linux-x86_64/* /usr/ && \
 rm -rf cmake-3.12.0-Linux-x86_64 && \
 rm -rf cmake.tar.gz

# install rustup
RUN curl -sSf https://static.rust-lang.org/rustup.sh -o rustup.sh &&\
ls&&\
 sh rustup.sh --disable-sudo

# show backtraces
ENV RUST_BACKTRACE 1

# set compiler
ENV CXX g++
ENV CC gcc

# show tools
RUN rustc -vV && \
cargo -V && \
gcc -v &&\
g++ -v &&\
cmake --version

# build parity
ADD . /build/parity
RUN cd parity&&\
    cargo build --release --verbose && \
	ls /build/parity/target/release/parity &&	\
	strip /build/parity/target/release/parity

RUN file /build/parity/target/release/parity

RUN mkdir -p /opt/parity && \
    chmod g+rwX /opt/parity

WORKDIR /opt/parity

EXPOSE 8080 8545 818

USER 1001

ENTRYPOINT ["/build/parity/target/release/parity"]


