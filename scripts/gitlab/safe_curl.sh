#!/usr/bin/env bash

set -eu

DATA=$1
ADDRESS=$2

CODE=$(curl -o out.txt -w '%{http_code}' --data $DATA $ADDRESS)
cat out.txt && rm out.txt
echo "\n"

if [[ $CODE -eq 200 ]]; then
	echo 'Pushed to updater service.';
elif [[ $CODE -eq 202 ]]; then
	echo 'Updater service ignored request.';
else
	echo 'Unable to push info to updater service.';
	exit 2
fi
