#!/bin/bash
set -e

# variables
UTCDATE=`date -u "+%Y%m%d-%H%M%S"`
PACKAGES=( "parity" )
BRANCH=$CI_BUILD_REF_NAME
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
echo "*** Setting up GitHub config for js-precompiled"
rm -rf ./.git
git init
setup_git_user

echo "*** Checking out $BRANCH branch"
git remote add origin $GIT_JS_PRECOMPILED
git fetch origin 2>$GITLOG
git checkout -b $BRANCH

echo "*** Committing compiled files for $UTCDATE"
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

echo "*** Merging remote"
git push origin HEAD:refs/heads/$BRANCH 2>$GITLOG
PRECOMPILED_HASH=`git rev-parse HEAD`

# move to root
cd ../..

echo "*** Setting up GitHub config for parity"
setup_git_user
git remote set-url origin $GIT_PARITY
git reset --hard origin/$BRANCH 2>$GITLOG

if [ "$BRANCH" == "master" ]; then
  cd js

  echo "*** Bumping package.json patch version"
  npm --no-git-tag-version version
  npm version patch

  echo "*** Building packages for npmjs"
  echo "$NPM_TOKEN" >> ~/.npmrc

  for PACKAGE in ${PACKAGES[@]}
  do
    echo "*** Building $PACKAGE"
    LIBRARY=$PACKAGE npm run ci:build:npm
    DIRECTORY=.npmjs/$PACKAGE

    echo "*** Publishing $PACKAGE from $DIRECTORY"
    cd $DIRECTORY
    npm publish --access public || true
    cd ../..
  done

  cd ..
fi

echo "*** Updating cargo parity-ui-precompiled#$PRECOMPILED_HASH"
git submodule update
sed -i "/^parity-ui-precompiled/ { s/branch = \".*\"/branch = \"$BRANCH\"/g; }" dapps/ui/Cargo.toml
cargo update -p parity-ui-precompiled
# --precise "$PRECOMPILED_HASH"

echo "*** Committing updated files"
git add js
git add dapps/ui/Cargo.toml
git add Cargo.lock
git commit -m "[ci skip] js-precompiled $UTCDATE"
git push origin HEAD:refs/heads/$BRANCH 2>$GITLOG

# back to root
echo "*** Release completed"
popd

# exit with exit code
exit 0
