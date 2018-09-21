FROM alpine:edge AS builder

# show backtraces
ENV RUST_BACKTRACE 1

RUN apk add --no-cache \
  build-base \
  cargo \
  cmake \
  eudev-dev \
  linux-headers \
  perl \
  rust

WORKDIR /parity
COPY . /parity
RUN cargo build --release --target x86_64-alpine-linux-musl --verbose
RUN strip target/x86_64-alpine-linux-musl/release/parity


FROM alpine:edge

# show backtraces
ENV RUST_BACKTRACE 1

RUN apk add --no-cache \
  libstdc++ \
  eudev-libs \
  libgcc

RUN addgroup -g 1000 parity \
  && adduser -u 1000 -G parity -s /bin/sh -D parity

USER parity

EXPOSE 5001 8080 8082 8083 8545 8546 8180 30303/tcp 30303/udp

WORKDIR /home/parity

RUN mkdir -p /home/parity/.local/share/io.parity.ethereum/
COPY --chown=parity:parity --from=builder /parity/target/x86_64-alpine-linux-musl/release/parity ./

ENTRYPOINT ["./parity"]
