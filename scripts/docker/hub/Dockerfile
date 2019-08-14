FROM ubuntu:xenial

# metadata
ARG VCS_REF
ARG BUILD_DATE

LABEL io.parity.image.authors="devops-team@parity.io" \
	io.parity.image.vendor="Parity Technologies" \
	io.parity.image.title="parity/parity" \
	io.parity.image.description="Parity Ethereum. The Fastest and most Advanced Ethereum Client." \
	io.parity.image.source="https://github.com/paritytech/parity-ethereum/blob/${VCS_REF}/\
scripts/docker/hub/Dockerfile" \
	io.parity.image.documentation="https://wiki.parity.io/Parity-Ethereum" \
	io.parity.image.revision="${VCS_REF}" \
	io.parity.image.created="${BUILD_DATE}"

# show backtraces
ENV RUST_BACKTRACE 1

# install tools and dependencies
RUN set -eux; \
	apt-get update; \
	apt-get install -y --no-install-recommends \
		file curl jq; \
# apt cleanup
	apt-get autoremove -y; \
	apt-get clean; \
	rm -rf /tmp/* /var/tmp/* /var/lib/apt/lists/*; \
# add user
	groupadd -g 1000 parity; \
	useradd -m -u 1000 -g parity -s /bin/sh parity

WORKDIR /home/parity

# add parity-ethereum binary to docker image
COPY artifacts/x86_64-unknown-linux-gnu/parity /bin/parity
COPY tools/check_sync.sh /check_sync.sh

# switch to user parity here
USER parity

# check if executable works in this container
RUN parity --version

EXPOSE 5001 8080 8082 8083 8545 8546 8180 30303/tcp 30303/udp

ENTRYPOINT ["/bin/parity"]
