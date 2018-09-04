#!/bin/bash

set -e # fail on any error
set -u # treat unset variables as error

clone_repos() {
    echo "__________Clone repos__________"
    git clone https://github.com/parity-js/jsonrpc.git jsonrpc
    git clone https://github.com/paritytech/wiki.git wiki
}

build_docs() {
    echo "__________Build docs__________"
    npm install
    npm run build:markdown
}

update_wiki_docs() {
    echo "__________Update WIKI docs__________"
    for file in $(ls jsonrpc/docs); do
        module_name=${file:0:-3}
        mv jsonrpc/docs/$file wiki/JSONRPC-$module_name-module.md
    done
}

setup_git() {
    echo "__________Set github__________"
    git config user.email "devops@parity.com"
    git config user.name "Devops Parity"
}

commit_files() {
    echo "__________Commit files__________"
    git checkout -b rpcdoc-update-${CI_COMMIT_REF_NAME}
    git add .
    git commit -m "Update docs to ${CI_COMMIT_REF_NAME}"
    git tag -a "${CI_COMMIT_REF_NAME}"
}

upload_files() {
    echo "__________Upload files__________"
    git push --tags
}

setup_git
clone_repos
cp -r parity/ jsonrpc/.parity/
cd jsonrpc
build_docs
cd ..
update_wiki_docs
cd wiki
commit_files
upload_files
