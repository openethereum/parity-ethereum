FROM ubuntu:16.04
LABEL maintainer="Parity Technologies <devops@parity.io>"

RUN mkdir /parity
WORKDIR /parity

RUN apt install libssl1.0.0 libudev1
COPY artifacts/parity /parity/parity

EXPOSE 8080 8545 8180
ENTRYPOINT ["/parity/parity"]
