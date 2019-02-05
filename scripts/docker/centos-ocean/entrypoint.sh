#!/bin/bash

if [ "${CONFIGURE_ACL}" = "true" ]; then
  if [ ! -z "${ACL_CONTRACT_ADDRESS}" ]; then
    acl_contract="${ACL_CONTRACT_ADDRESS}"
  else
    echo "Waiting for contracts to be generated..."
    while [ ! -f "/contracts/ready" ] || [ ! -f "/contracts/AccessConditions.${NETWORK}.json" ]; do
      sleep 2
    done
    acl_contract=$(cat /contracts/AccessConditions.${NETWORK}.json | jq -r .address)
  fi
  echo "ACL Contract Address: ${acl_contract}"
  sed -i -e "/acl_contract = .*/c acl_contract = \"${acl_contract:2}\"" /etc/parity/secretstore/config.toml
else
  sed -i -e "/acl_contract = .*/c acl_contract = \"none\"" /etc/parity/secretstore/config.toml
fi

/opt/parity/parity "$@"
