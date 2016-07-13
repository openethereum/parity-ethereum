#!/bin/sh
# generate documentation only for partiy and ethcore libraries

. ./scripts/targets.sh

cargo doc --no-deps --verbose $TARGETS &&
	echo '<meta http-equiv=refresh content=0;url=ethcore/index.html>' > target/doc/index.html
