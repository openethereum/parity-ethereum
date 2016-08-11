#!/bin/sh

PROTECTED_BRANCH='master'
CURRENT_BRANCH=$(git symbolic-ref HEAD | sed -e 's,.*/\(.*\),\1,')

if [ $PROTECTED_BRANCH = $CURRENT_BRANCH ];
then
	echo "Direct commits to the branch master are not allowed"
	exit 1
fi
