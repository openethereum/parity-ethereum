#!/bin/bash
set -e

# variables
PVER="1-9"
UTCDATE=`date -u "+%Y%m%d-%H%M%S"`
BRANCH=$CI_BUILD_REF_NAME
GIT_PARITY="https://${GITHUB_JS_PRECOMPILED}:@github.com/paritytech/parity.git"

echo "*** [cargo] Setting up GitHub config for parity"
git config push.default simple
git config merge.ours.driver true
git config user.email "$GITHUB_EMAIL"
git config user.name "GitLab Build Bot"
git remote set-url origin $GIT_PARITY > /dev/null 2>&1
git checkout $BRANCH
git reset --hard origin/$BRANCH 2>/dev/null
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
sed -i "/^parity-ui-precompiled/ { s/git = \".*\"/git = \"https:\/\/github.com\/js-dist-paritytech\/parity-$BRANCH-$PVER-shell.git\"/g; }" dapps/ui/Cargo.toml
cargo update -p parity-ui-precompiled

echo "*** [cargo] Updating cargo parity-ui-old-precompiled"
sed -i "/^parity-ui-old-precompiled/ { s/git = \".*\"/git = \"https:\/\/github.com\/js-dist-paritytech\/parity-$BRANCH-$PVER-v1.git\"/g; }" dapps/ui/Cargo.toml
cargo update -p parity-ui-old-precompiled

echo "*** [cargo] Committing updated files"
git add dapps/ui/Cargo.toml
git add Cargo.lock
git commit -m "[ci skip] js-precompiled $UTCDATE"
git push origin HEAD:refs/heads/$BRANCH 2>/dev/null

# exit with exit code
exit 0
