#!/bin/bash

# change into the build directory
pushd `dirname $0`
cd ../.build

# variables
UTCDATE=`date -u "+%Y%m%d-%H%M%S"`

# init git
rm -rf ./.git
git init

# our user details
git config push.default simple
git config merge.ours.driver true
git config user.email "jaco+gitlab@ethcore.io"
git config user.name "GitLab Build Bot"

# add local files and send it up
git remote add origin https://${GITHUB_JS_PRECOMPILED}:@github.com/ethcore/js-precompiled.git
git fetch origin
git checkout -b $CI_BUILD_REF_NAME
git add .
git commit -m "$UTCDATE [compiled]"
git merge origin/$CI_BUILD_REF_NAME -X ours --commit -m "$UTCDATE [release]"
git push origin $CI_BUILD_REF_NAME

# back to root
popd

# exit with exit code
exit 0
