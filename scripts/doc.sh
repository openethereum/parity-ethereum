#!/bin/sh
# generate documentation only for partiy and ethcore libraries

cargo doc --no-deps --verbose --all --exclude parity-ipfs-api &&
	echo '<meta http-equiv=refresh content=0;url=ethcore/index.html>' > target/doc/index.html
