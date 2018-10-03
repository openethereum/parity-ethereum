FROM ubuntu:xenial
MAINTAINER Parity Technologies <devops@parity.io>
#set ENVIROMENT
ARG TARGET
ENV TARGET ${TARGET}

# install tools and dependencies
RUN apt update && apt install -y --no-install-recommends openssl libudev-dev file

# show backtraces
ENV RUST_BACKTRACE 1

#cleanup Docker image
RUN apt autoremove -y
RUN apt clean -y
RUN rm -rf /tmp/* /var/tmp/* /var/lib/apt/lists/*

RUN groupadd -g 1000 parity \
  && useradd -m -u 1000 -g parity -s /bin/sh parity

USER parity

WORKDIR /home/parity

ENV PATH "~/bin:${PATH}"

#add TARGET to docker image
COPY artifacts/x86_64-unknown-linux-gnu/$TARGET ./bin/$TARGET

# Build a shell script because the ENTRYPOINT command doesn't like using ENV
RUN echo "#!/bin/bash \n ${TARGET} \$@" > ./entrypoint.sh
RUN chmod +x ./entrypoint.sh

# setup ENTRYPOINT
EXPOSE 5001 8080 8082 8083 8545 8546 8180 30303/tcp 30303/udp
ENTRYPOINT ["./entrypoint.sh"]
