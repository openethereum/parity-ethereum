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

#add TARGET to docker image
COPY artifacts/$TARGET /usr/bin/$TARGET

# Build a shell script because the ENTRYPOINT command doesn't like using ENV
RUN echo "#!/bin/bash \n ${TARGET} \$@" > ./entrypoint.sh
RUN chmod +x ./entrypoint.sh

# setup ENTRYPOINT
EXPOSE 8080 8545 8180
ENTRYPOINT ["./entrypoint.sh"]
