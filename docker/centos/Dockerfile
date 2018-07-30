FROM centos:latest

RUN mkdir -p /opt/parity/data && \
    chmod g+rwX /opt/parity/data && \
    mkdir -p /opt/parity/release

COPY parity/parity /opt/parity/release

WORKDIR /opt/parity/data

# exposing default ports
#
#           secret
#      ipfs store     ui   rpc  ws   listener  discovery
#      ↓    ↓         ↓    ↓    ↓    ↓         ↓
EXPOSE 5001 8082 8083 8180 8545 8546 30303/tcp 30303/udp

# switch to non-root user
USER 1001

#if no base path provided, assume it's current workdir
CMD ["--base-path","."]
ENTRYPOINT ["/opt/parity/release/parity"]  


   


