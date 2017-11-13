#!/bin/bash
set -e

# variables
UTCDATE=`date -u "+%Y%m%d-%H%M%S"`
PRECOMPILED_BRANCH="v1"
GIT_JS_PRECOMPILED="https://${GITHUB_JS_PRECOMPILED}:@github.com/paritytech/js-precompiled.git"

# setup the git user defaults for the current repo
function setup_git_user {
  git config push.default simple
  git config merge.ours.driver true
  git config user.email "$GITHUB_EMAIL"
  git config user.name "GitLab Build Bot"
}

# change into the build directory
BASEDIR=`dirname $0`
GITLOG=.git-release.log
pushd $BASEDIR
cd ../.dist

# add local files and send it up
echo "*** [v1 precompiled] Setting up GitHub config for js-precompiled"
rm -rf ./.git
git init
setup_git_user

echo "*** [v1 precompiled] Checking out $PRECOMPILED_BRANCH branch"
git remote add origin $GIT_JS_PRECOMPILED
git fetch origin 2>$GITLOG
git checkout -b $PRECOMPILED_BRANCH

echo "*** [v1 precompiled] Committing compiled files for $UTCDATE"
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

echo "*** [v1 precompiled] Merging remote"
git push origin HEAD:refs/heads/$PRECOMPILED_BRANCH 2>$GITLOG

# move to root
popd

# exit with exit code
exit 0
