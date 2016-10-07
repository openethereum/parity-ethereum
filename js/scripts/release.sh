#!/bin/bash

# change into the js directory (one down from scripts)
pushd `dirname $0`
cd ../build

# variables
UTCDATE=`date -u "+%Y%m%d-%H%M%S"`

# init git
rm -rf ./.git
git init

# our user details
git config push.default simple
git config user.email "jaco+gitlab@ethcore.io"
git config user.name "GitLab Build Bot"

# add local files and send it up
git remote add origin https://${GITHUB_JS_PRECOMPILED}:@github.com/ethcore/js-precompiled.git
git fetch
git checkout master
git add .
git commit -m "$UTCDATE"
git push origin master --force

# back to root
popd

# exit with exit code
exit 0
