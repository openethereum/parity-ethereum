# ethcore

[![Build Status][travis-image]][travis-url] [![Coverage Status][coveralls-image]][coveralls-url]

[travis-image]: https://travis-ci.com/ethcore/parity.svg?token=DMFvZu71iaTbUYx9UypX&branch=master
[travis-url]: https://travis-ci.com/ethcore/parity
[coveralls-image]: https://coveralls.io/repos/github/ethcore/parity/badge.svg?branch=master&t=Fk0OuQ
[coveralls-url]: https://coveralls.io/r/ethcore/parity?branch=master


### Building from source

##### 14.04 Trusty

```bash
# install rocksdb
add-apt-repository "deb http://ppa.launchpad.net/giskou/librocksdb/ubuntu trusty main"
apt-get update
apt-get install -y --force-yes librocksdb

# install multirust
curl -sf https://raw.githubusercontent.com/brson/multirust/master/blastoff.sh | sh -s -- --yes

# install nightly and make it default
multirust update nightly && multirust default nightly

# export rust LIBRARY_PATH
LIBRARY_PATH /usr/local/lib

# download and build parity
git clone https://github.com/ethcore/parity
cd parity
cargo build --release
```

##### OSX

```bash
# install rocksdb
brew update
brew install rocksdb

# install multirust
curl -sf https://raw.githubusercontent.com/brson/multirust/master/blastoff.sh | sh -s -- --yes

# install nightly and make it default
multirust update nightly && multirust default nightly

# export rust LIBRARY_PATH
LIBRARY_PATH /usr/local/lib

# download and build parity
git clone https://github.com/ethcore/parity
cd parity
cargo build --release
```
