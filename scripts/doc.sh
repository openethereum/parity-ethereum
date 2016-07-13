#!/bin/sh
# generate documentation only for partiy and ethcore libraries

. ./scripts/targets.sh

cargo doc --no-deps --verbose --no-default-features $TARGETS &&
	echo '<meta http-equiv=refresh content=0;url=ethcore/index.html>' > target/doc/index.html
