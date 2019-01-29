FROM ubuntu:xenial
LABEL MAINTAINER="Parity Technologies devops-team@parity.io"

# install tools and dependencies
RUN apt update && apt install -y --no-install-recommends openssl libudev-dev file curl jq

# show backtraces
ENV RUST_BACKTRACE 1

# cleanup Docker image
RUN apt autoremove -y \
  && apt clean -y \
  && rm -rf /tmp/* /var/tmp/* /var/lib/apt/lists/*

RUN groupadd -g 1000 parity \
  && useradd -m -u 1000 -g parity -s /bin/sh parity

# add parity to docker image
COPY artifacts/x86_64-unknown-linux-gnu/parity /bin/parity
RUN echo "#!/bin/bash \n /bin/parity \$@" > ./entrypoint.sh
COPY scripts/docker/hub/check_sync.sh /check_sync.sh

# switch to user parity here
USER parity

VOLUME [ "/home/parity/.local/share/io.parity.ethereum" ]
EXPOSE 5001 8080 8082 8083 8545 8546 8180 30303/tcp 30303/udp
# setup ENTRYPOINT
ENTRYPOINT ["/bin/parity"]
