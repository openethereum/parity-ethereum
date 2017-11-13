#!/bin/bash
set -e

# variables
UTCDATE=`date -u "+%Y%m%d-%H%M%S"`
BRANCH=$CI_BUILD_REF_NAME
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
echo "*** [v2 precompiled] Setting up GitHub config"
rm -rf ./.git
git init
setup_git_user

echo "*** [v2 precompiled] Checking out $BRANCH branch"
git remote add origin $GIT_JS_PRECOMPILED
git fetch origin 2>$GITLOG
git checkout -b $BRANCH

echo "*** [v2 precompiled] Committing compiled files for $UTCDATE"
mv build ../build.new
git add .
git commit -m "$UTCDATE [update]"
git merge origin/$BRANCH -X ours --commit -m "$UTCDATE [merge]"
git rm -r build
rm -rf build
git commit -m "$UTCDATE [cleanup]"
mv ../build.new build
git add .
git commit -m "$UTCDATE [release]"

echo "*** [v2 precompiled] Merging remote"
git push origin HEAD:refs/heads/$BRANCH 2>$GITLOG

popd

# exit with exit code
exit 0
