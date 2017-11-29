#!/bin/bash
set -e

# variables
PVER="1-9"
PTYPE="shell"
UTCDATE=`date -u "+%Y%m%d-%H%M%S"`
PRE_REPO="js-dist-paritytech/parity-${CI_BUILD_REF_NAME}-${PVER}-${PTYPE}.git"
PRE_REPO_TOKEN="https://${GITHUB_JS_PRECOMPILED}:@github.com/${PRE_REPO}"
BASEDIR=`dirname $0`

pushd $BASEDIR/..

echo "*** [$PRE_REPO] Cloning repo"
rm -rf precompiled
git clone https://github.com/$PRE_REPO precompiled
cd precompiled
git config push.default simple
git config merge.ours.driver true
git config user.email "$GITHUB_EMAIL"
git config user.name "GitLab Build Bot"
git remote set-url origin $PRE_REPO_TOKEN > /dev/null 2>&1

echo "*** [$PRE_REPO] Copying build"
rm -rf build
cp -rf ../.dist/build .

echo "*** [$PRE_REPO] Adding to git"
echo "$UTCDATE" >README.md
git add .
git commit -m "$UTCDATE"

echo "*** [$PRE_REPO] Pushing upstream"
git push --quiet origin HEAD:refs/heads/master > /dev/null 2>&1

cd ..
rm -rf .dist .build .happypack precompiled
popd

# exit with exit code
exit 0
