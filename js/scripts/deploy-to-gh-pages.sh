#!/bin/bash
# See https://medium.com/@nthgergo/publishing-gh-pages-with-travis-ci-53a8270e87db
set -o errexit

SOURCE_BRANCH="master"
TARGET_BRANCH="gh-pages"
GIT_USER_EMAIL="admin@travis-ci.org"
GIT_USER_NAME="Travis CI"
SHA=$(git rev-parse --verify HEAD) # reference to commit deployed to TARGET_BRANCH
COMMIT_MSG="Deploy to Github Pages: ${SHA}"

# Don't deploy if
# 1. Pull request
# 2. Not target branch
# 3. Forked repo

if [[ ("$TRAVIS_PULL_REQUEST" != "false") || ("$TRAVIS_BRANCH" != "$SOURCE_BRANCH") || ("$TRAVIS_REPO_SLUG" != "$REPO_SLUG") ]]; then
    echo 'Not deploying';
    exit 0
fi


# config
git config --global user.email "$GIT_USER_EMAIL"
git config --global user.name "$GIT_USER_NAME"

# deploy
cd lib
git init
git add .
git commit -m "$COMMIT_MSG"

git push -f --quiet "https://${GITHUB_TOKEN}@github.com/${REPO_SLUG}.git" $SOURCE_BRANCH:$TARGET_BRANCH > /dev/null 2>&1
