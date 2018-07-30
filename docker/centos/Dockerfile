FROM centos:latest

WORKDIR /opt/parity/

RUN mkdir -p /opt/parity/data && \
    chmod g+rwX /opt/parity/data && \
    mkdir -p /opt/parity/release

COPY parity/parity ./release
CMD ["/opt/parity/release/parity"]  


   


