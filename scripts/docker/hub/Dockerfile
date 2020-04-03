FROM ubuntu:xenial

# metadata
ARG VCS_REF
ARG BUILD_DATE

LABEL openethereum.image.authors="devops-team@parity.io" \
	openethereum.image.vendor="OpenEthereum project" \
	openethereum.image.title="openethereum/openethereum" \
	openethereum.image.description="Fast and feature-rich multi-network Ethereum client." \
	openethereum.image.source="https://github.com/openethereum/openethereum/blob/${VCS_REF}/\
	scripts/docker/hub/Dockerfile" \
	openethereum.image.documentation="https://wiki.parity.io/Parity-Ethereum" \
	openethereum.image.revision="${VCS_REF}" \
	openethereum.image.created="${BUILD_DATE}"

# show backtraces
ENV RUST_BACKTRACE 1

# install tools and dependencies
RUN set -eux; \
	apt-get update; \
	apt-get install -y --no-install-recommends \
	file curl jq ca-certificates; \
	# apt cleanup
	apt-get autoremove -y; \
	apt-get clean; \
	update-ca-certificates; \
	rm -rf /tmp/* /var/tmp/* /var/lib/apt/lists/*; \
	# add user
	groupadd -g 1000 openethereum; \
	useradd -m -u 1000 -g openethereum -s /bin/sh openethereum

WORKDIR /home/openethereum

# add openethereum binary to docker image
COPY artifacts/x86_64-unknown-linux-gnu/openethereum /bin/openethereum
COPY tools/check_sync.sh /check_sync.sh

# switch to user openethereum here
USER openethereum

# check if executable works in this container
RUN openethereum --version

EXPOSE 5001 8080 8082 8083 8545 8546 8180 30303/tcp 30303/udp

ENTRYPOINT ["/bin/openethereum"]
