#!/bin/bash
set -e

# setup the git user defaults for the current repo
function setup_git_user {
  git config push.default simple
  git config merge.ours.driver true
  git config user.email "jaco+gitlab@ethcore.io"
  git config user.name "GitLab Build Bot"
}

# change into the build directory
BASEDIR=`dirname $0`
GITLOG=./.git/gitcommand.log
pushd $BASEDIR
cd ../.dist

# variables
UTCDATE=`date -u "+%Y%m%d-%H%M%S"`

# init git
rm -rf ./.git
git init

# add local files and send it up
echo "Setting up GitHub config for js-precompiled"
setup_git_user

echo "Checking out $CI_BUILD_REF_NAME branch"
git remote add origin https://${GITHUB_JS_PRECOMPILED}:@github.com/ethcore/js-precompiled.git
git fetch origin 2>$GITLOG
git checkout -b $CI_BUILD_REF_NAME

echo "Committing compiled files for $UTCDATE"
git add .
git commit -m "$UTCDATE"

echo "Merging remote"
git merge origin/$CI_BUILD_REF_NAME -X ours --commit -m "$UTCDATE [release]"
git push origin HEAD:refs/heads/$CI_BUILD_REF_NAME 2>$GITLOG
PRECOMPILED_HASH=$(git rev-parse HEAD)

echo "Remote updated with [$PRECOMPILED_HASH]"

# back to root
popd

echo "Setting up GitHub config for parity"
setup_git_user
git remote set-url origin https://${GITHUB_JS_PRECOMPILED}:@github.com/ethcore/parity.git
git reset --hard origin/$CI_BUILD_REF_NAME 2>$GITLOG

echo "Updating cargo package parity-ui-precompiled"
cargo update -p parity-ui-precompiled --precise $PRECOMPILED_HASH

echo "Committing updated files"
git add . || true
git commit -m "[ci skip] js-precompiled $UTCDATE"
git push origin HEAD:refs/heads/$CI_BUILD_REF_NAME 2>$GITLOG

# exit with exit code
exit 0
