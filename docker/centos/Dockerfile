FROM centos:latest

ADD . /build/parity

# show backtraces & set compiler
ENV RUST_BACKTRACE 1
ENV CXX g++
ENV CC gcc

# install tools and dependencies, build parity, set folder permissions
RUN yum -y update && \
    yum install -y systemd-devel git make gcc-c++ gcc file binutils && \
    curl -L "https://cmake.org/files/v3.12/cmake-3.12.0-Linux-x86_64.tar.gz" -o cmake.tar.gz && \
    tar -xzf cmake.tar.gz && \
    cp -r cmake-3.12.0-Linux-x86_64/* /usr/ && \
    rm -rf cmake-3.12.0-Linux-x86_64 && \
    rm -rf cmake.tar.gz && \
    curl -sSf https://static.rust-lang.org/rustup.sh -o rustup.sh && \
    sh rustup.sh --disable-sudo && \
    rustc -vV && \
	cargo -V && \
	gcc -v && \
	g++ -v && \
    cmake --version && \
    cd build/parity && \
    cargo build --release --verbose && \
	ls /build/parity/target/release/parity && \
	strip /build/parity/target/release/parity && \
    file /build/parity/target/release/parity && \
    mkdir -p /opt/parity/target/release && \
	cp -R /build/parity/target/release /opt/parity/target && \
	rm -r /build && \
    mkdir -p /opt/parity/data && \
    chmod g+rwX /opt/parity/data && \
    yum clean all -y && \
    rm -rf /var/cache/yum

WORKDIR /opt/parity/data

EXPOSE 8080 8545 8180

# switch to non-root user
USER 1001

ENTRYPOINT ["/opt/parity/target/release/parity"]
