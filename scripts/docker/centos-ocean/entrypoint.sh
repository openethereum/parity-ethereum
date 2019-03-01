#!/bin/bash

if [ "${CONFIGURE_ACL}" = "true" ]; then
  if [ ! -z "${ACL_CONTRACT_ADDRESS}" ]; then
    acl_contract="${ACL_CONTRACT_ADDRESS}"
  else
    echo "Waiting for contracts to be generated..."
    while [ ! -f "/contracts/ready" ]; do
      sleep 2
    done
    while [ ! -f "/contracts/AccessSecretStoreCondition.${NETWORK}.json" ] && [ ! -f "/contracts/AccessConditions.${NETWORK}.json" ]; do
      sleep 2
    done
    if [ -f "/contracts/AccessSecretStoreCondition.${NETWORK}.json" ]; then
      acl_contract=$(cat /contracts/AccessSecretStoreCondition.${NETWORK}.json | jq -r .address)
    else
      acl_contract=$(cat /contracts/AccessConditions.${NETWORK}.json | jq -r .address)
    fi
  fi
  echo "ACL Contract Address: ${acl_contract}"
  sed -i -e "/acl_contract = .*/c acl_contract = \"${acl_contract:2}\"" /etc/parity/secretstore/config.toml
else
  sed -i -e "/acl_contract = .*/c acl_contract = \"none\"" /etc/parity/secretstore/config.toml
fi

/opt/parity/parity "$@"
