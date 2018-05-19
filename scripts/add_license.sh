#!/bin/sh

PAT="^// Copyright.*If not, see <http://www.gnu.org/licenses/>\.$"

for f in $(find . -name '*.rs'); do
	HEADER=$(head -16 $f)
	if [[ $HEADER =~ $PAT ]]; then
		BODY=$(tail -n +17 $f)
		cat license_header > temp
		echo "$BODY" >> temp
		mv temp $f
	else
		echo "$f was missing header" 
		cat license_header $f > temp
		mv temp $f
	fi
done
