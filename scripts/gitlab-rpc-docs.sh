#!/usr/bin/env bash

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

setup_git() {
    git config user.email "devops@parity.com"
    git config user.name "Devops Parity"
}

commit_files() {
    git checkout -b rpcdoc-update-${CI_COMMIT_REF_NAME}
    git commit .
    git commit -m "Update docs to ${CI_COMMIT_REF_NAME}"
    git tag -a "${CI_COMMIT_REF_NAME}"
}

upload_files() {
    git push --tags
}

setup_git
clone_repos
cp parity jsonrpc/.parity
cd jsonrpc
build_docs
cd ..
update_wiki_docs
cd wiki
commit_files
upload_files
