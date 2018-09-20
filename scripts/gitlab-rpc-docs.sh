#!/usr/bin/env bash

set -e # fail on errors



clone_repos() {
    git clone https://github.com/parity-js/jsonrpc.git jsonrpc
    git clone https://github.com/paritytech/wiki.git wiki
}

build_docs() {
    npm install
    npm run build:markdown
}

update_wiki_docs() {
    for file in $(ls jsonrpc/docs); do
        module_name=${file:0:-3}
        mv jsonrpc/docs/$file wiki/JSONRPC-$module_name-module.md
    done
}

set_remote_wiki() {
    git config remote.origin.url "https://${GITHUB_TOKEN}@github.com/paritytech/wiki.git"
}

setup_git() {
    git config --global user.email "devops@parity.com"
    git config --global user.name "Devops Parity"
}

commit_files() {
    git checkout -b rpcdoc-update-${CI_COMMIT_REF_NAME}
    git add .
    git commit -m "Update docs to ${CI_COMMIT_REF_NAME}"
    git tag -a "${CI_COMMIT_REF_NAME}" -m "Updated to ${CI_COMMIT_REF_NAME}"
}

upload_files() {
    git push --tags
}

RPC_TRAITS_DIR="rpc/src/v1/traits"

setup_git
cd ..
clone_repos
mkdir -p "jsonrpc/.parity/$RPC_TRAITS_DIR"
cp $RPC_TRAITS_DIR/*.rs "jsonrpc/.parity/$RPC_TRAITS_DIR"
cd jsonrpc
build_docs
cd ..
update_wiki_docs
cd wiki
set_remote_wiki
commit_files
upload_files
