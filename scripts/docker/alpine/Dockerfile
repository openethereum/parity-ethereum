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
  rust \
  clang-dev \
  llvm-dev

WORKDIR /parity
COPY . /parity
ENV C=clang CXX=clang++
RUN cargo build --release --features final --target x86_64-alpine-linux-musl --verbose
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

EXPOSE 8080 8545 8180

WORKDIR /home/parity

RUN mkdir -p /home/parity/.local/share/io.parity.ethereum/
COPY --chown=parity:parity --from=builder /parity/target/x86_64-alpine-linux-musl/release/parity ./

ENTRYPOINT ["./parity"]
