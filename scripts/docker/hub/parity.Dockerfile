ARG FROM_VERSION

FROM parity/ethereum:${FROM_VERSION}

# metadata
ARG VCS_REF
ARG BUILD_DATE

LABEL io.parity.image.authors="devops-team@parity.io" \
	io.parity.image.vendor="Parity Technologies" \
	io.parity.image.title="parity/parity" \
	io.parity.image.description="Parity Ethereum. Deprecated docker image." \
	io.parity.image.source="https://github.com/paritytech/parity-ethereum/blob/${VCS_REF}/\
scripts/docker/hub/Dockerfile" \
	io.parity.image.documentation="https://wiki.parity.io/Parity-Ethereum" \
	io.parity.image.revision="${VCS_REF}" \
	io.parity.image.created="${BUILD_DATE}"

COPY tools/entrypoint.sh /entrypoint.sh

# check if executable works in this container
RUN parity --version

ENTRYPOINT ["/entrypoint.sh"]
