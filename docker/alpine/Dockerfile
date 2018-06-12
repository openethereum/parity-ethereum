FROM alpine:edge

WORKDIR /build

# install tools and dependencies
RUN apk add --no-cache gcc musl-dev pkgconfig g++ make curl \
						eudev-dev rust cargo git file binutils \
						libusb-dev linux-headers perl

# show backtraces
ENV RUST_BACKTRACE 1

# show tools
RUN rustc -vV && \
cargo -V && \
gcc -v &&\
g++ -v

# build parity
ADD . /build/parity
RUN cd parity && \
	cargo build --release --verbose && \
	ls /build/parity/target/release/parity && \
	strip /build/parity/target/release/parity

RUN file /build/parity/target/release/parity

EXPOSE 8080 8545 8180
ENTRYPOINT ["/build/parity/target/release/parity"]
