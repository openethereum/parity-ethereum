#!/bin/bash
set -e

# variables
UTCDATE=`date -u "+%Y%m%d-%H%M%S"`
BRANCH=$CI_BUILD_REF_NAME
GIT_PARITY="https://${GITHUB_JS_PRECOMPILED}:@github.com/paritytech/parity.git"

# setup the git user defaults for the current repo
function setup_git_user {
  git config push.default simple
  git config merge.ours.driver true
  git config user.email "$GITHUB_EMAIL"
  git config user.name "GitLab Build Bot"
}

# change into the build directory
BASEDIR=`dirname $0`
pushd $BASEDIR

echo "*** [cargo] Setting up GitHub config for parity"
setup_git_user
git remote set-url origin $GIT_PARITY
git reset --hard origin/$BRANCH 2>$GITLOG
git submodule update

if [ "$BRANCH" == "master" ]; then
  cd js

  echo "*** [cargo] Bumping package.json patch version"
  npm --no-git-tag-version version
  npm version patch

  cd ..

  git add js
fi

echo "*** [cargo] Updating cargo parity-ui-precompiled"
sed -i "/^parity-ui-precompiled/ { s/branch = \".*\"/branch = \"$BRANCH\"/g; }" dapps/ui/Cargo.toml
cargo update -p parity-ui-precompiled
cargo update -p parity-ui-old-precompiled

echo "*** [cargo] Committing updated files"
git add dapps/ui/Cargo.toml
git add Cargo.lock
git commit -m "[ci skip] js-precompiled $UTCDATE"
git push origin HEAD:refs/heads/$BRANCH 2>$GITLOG

# back to root
echo "*** [cargo] Release completed"
popd

# exit with exit code
exit 0
