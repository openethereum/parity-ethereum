FROM centos:latest

RUN mkdir -p /opt/openethereum/data && \
    chmod g+rwX /opt/openethereum/data && \
    mkdir -p /opt/openethereum/release

COPY openethereum/openethereum /opt/openethereum/release

WORKDIR /opt/openethereum/data

# exposing default ports
#
#      secret
#      store     ui   rpc  ws   listener  discovery
#      ↓         ↓    ↓    ↓    ↓         ↓
EXPOSE 8082 8083 8180 8545 8546 30303/tcp 30303/udp

# switch to non-root user
USER 1001

#if no base path provided, assume it's current workdir
CMD ["--base-path","."]
ENTRYPOINT ["/opt/openethereum/release/openethereum"]  
