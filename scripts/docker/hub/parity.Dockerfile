ARG FROM_VERSION

FROM parity/ethereum:${FROM_VERSION}

LABEL io.parity.image.description="Parity Ethereum. Deprecated docker image. Please use `parity/ethereum`."

COPY tools/entrypoint.sh /entrypoint.sh

ENTRYPOINT ["/entrypoint.sh"]
