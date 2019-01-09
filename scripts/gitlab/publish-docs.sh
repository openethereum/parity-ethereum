#!/bin/bash

set -e # fail on any error
set -u # treat unset variables as error

clone_repos() {
    echo "__________Clone repos__________"
    git clone https://github.com/parity-js/jsonrpc.git jsonrpc
    git clone https://github.com/paritytech/wiki.git wiki
    git clone https://github.com/paritytech/parity-config-generator
}

build_docs() {
    echo "__________Build docs__________"
    npm install
    npm run build:markdown
}

build_config() {
    echo "_______Build config docs______"
    yarn install
    AUTOGENSCRIPT=1 yarn generate-docs
}

update_wiki_docs() {
    echo "__________Update WIKI docs__________"
    for file in $(ls jsonrpc/docs); do
        module_name=${file:0:-3}
        mv jsonrpc/docs/$file wiki/JSONRPC-$module_name-module.md
    done
    mv parity-config-generator/docs/config.md wiki/Configuring-Parity-Ethereum.md
}

setup_git() {
    echo "__________Set github__________"
    git config --global user.email "devops@parity.com"
    git config --global user.name "Devops Parity"
}

set_remote_wiki() {
    git config remote.origin.url "https://${GITHUB_TOKEN}@github.com/paritytech/wiki.git"
}

commit_files() {
    echo "__________Commit files__________"
    git checkout -b rpcdoc-update-${SCHEDULE_TAG:-${CI_COMMIT_REF_NAME}}
    git add .
    git commit -m "Update docs to ${SCHEDULE_TAG:-${CI_COMMIT_REF_NAME}}"
    git tag -a -f "${SCHEDULE_TAG:-${CI_COMMIT_REF_NAME}}" -m "Update RPC and config docs to ${SCHEDULE_TAG:-${CI_COMMIT_REF_NAME}}"
}

upload_files() {
    echo "__________Upload files__________"
    git push -q origin HEAD
    git push -q -f --tags
}

RPC_TRAITS_DIR="rpc/src/v1/traits"

setup_git
clone_repos
mkdir -p "jsonrpc/.parity/$RPC_TRAITS_DIR"
cp $RPC_TRAITS_DIR/*.rs "jsonrpc/.parity/$RPC_TRAITS_DIR"
cd jsonrpc
build_docs
cd ..
cd parity-config-generator
build_config
cd ..
update_wiki_docs
cd wiki
set_remote_wiki
commit_files
upload_files
