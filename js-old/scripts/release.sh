#!/bin/bash
set -e

# variables
UTCDATE=`date -u "+%Y%m%d-%H%M%S"`
PRECOMPILED_BRANCH="v1"
GIT_JS_PRECOMPILED="https://${GITHUB_JS_PRECOMPILED}:@github.com/paritytech/js-precompiled.git"
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
GITLOG=./.git/gitcommand.log
pushd $BASEDIR
cd ../.dist

# add local files and send it up
echo "*** [v1] Setting up GitHub config for js-precompiled"
rm -rf ./.git
git init
setup_git_user

echo "*** [v1] Checking out $PRECOMPILED_BRANCH branch"
git remote add origin $GIT_JS_PRECOMPILED
git fetch origin 2>$GITLOG
git checkout -b $PRECOMPILED_BRANCH

echo "*** [v1] Committing compiled files for $UTCDATE"
mv build ../build.new
git add .
git commit -m "$UTCDATE [update]"
git merge origin/$PRECOMPILED_BRANCH -X ours --commit -m "$UTCDATE [merge]"
git rm -r build
rm -rf build
git commit -m "$UTCDATE [cleanup]"
mv ../build.new build
git add .
git commit -m "$UTCDATE [release]"

echo "*** [v1] Merging remote"
git push origin HEAD:refs/heads/$PRECOMPILED_BRANCH 2>$GITLOG
PRECOMPILED_HASH=`git rev-parse HEAD`

# move to root
cd ../..

echo "*** [v1] Setting up GitHub config for parity"
setup_git_user
git remote set-url origin $GIT_PARITY
git reset --hard origin/$BRANCH 2>$GITLOG

echo "*** [v1] Updating cargo parity-ui-old-precompiled#$PRECOMPILED_HASH"
git submodule update
# Not needed since $BRANCH is hardcoded
# sed -i "/^parity-ui-old-precompiled/ { s/branch = \".*\"/branch = \"$BRANCH\"/g; }" dapps/ui/Cargo.toml
cargo update -p parity-ui-old-precompiled
# --precise "$PRECOMPILED_HASH"

echo "*** [v1] Committing updated files"
git add dapps/ui/Cargo.toml
git add Cargo.lock
git commit -m "[ci skip] js-precompiled $UTCDATE"
git push origin HEAD:refs/heads/$BRANCH 2>$GITLOG

# back to root
echo "*** [v1] Release completed"
popd

# exit with exit code
exit 0
