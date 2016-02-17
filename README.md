# ethcore

[![Build Status][travis-image]][travis-url] [![Coverage Status][coveralls-image]][coveralls-url] [![Join the chat at https://gitter.im/trogdoro/xiki][gitter-image]][gitter-url]

[travis-image]: https://travis-ci.org/ethcore/parity.svg?branch=beta
[travis-url]: https://travis-ci.org/ethcore/parity
[coveralls-image]: https://coveralls.io/repos/github/ethcore/parity/badge.svg?branch=beta
[coveralls-url]: https://coveralls.io/github/ethcore/parity?branch=beta
[gitter-image]: https://badges.gitter.im/Join%20Chat.svg
[gitter-url]: https://gitter.im/ethcore/parity?utm_source=badge&utm_medium=badge&utm_campaign=pr-badge&utm_content=badge

[Documentation](http://ethcore.github.io/parity/ethcore/index.html)

### Building from source

##### Ubuntu 14.04, 15.04, 15.10

```bash
# install rocksdb
add-apt-repository ppa:ethcore/ethcore
apt-get update
apt-get install -y --force-yes librocksdb-dev

# install multirust
curl -sf https://raw.githubusercontent.com/brson/multirust/master/blastoff.sh | sh -s -- --yes

# install nightly and make it default
multirust update nightly-2016-02-14
multirust default nightly-2016-02-14

# download and build parity
git clone -b beta https://github.com/ethcore/parity
cd parity
cargo build --release
```

##### Other Linux

```bash
# install rocksdb
git clone --tag v4.1 --depth=1 https://github.com/facebook/rocksdb.git
cd rocksdb
make shared_lib 
sudo cp -a librocksdb.so* /usr/lib 
sudo ldconfig 
cd ..

# install rust nightly
curl -sf https://raw.githubusercontent.com/brson/multirust/master/blastoff.sh | sudo sh -s -- --yes

# install nightly and make it default
sudo multirust update nightly-2016-02-14
sudo multirust default nightly-2016-02-14

# download and build parity
git clone -b beta https://github.com/ethcore/parity
cd parity
cargo build --release
```

##### OSX with Homebrew

```bash
# install rocksdb && multirust
brew update
brew install rocksdb
brew install multirust

# install nightly and make it default
multirust update nightly-2016-02-14 && multirust default nightly-2016-02-14

# download and build parity
git clone -b beta https://github.com/ethcore/parity
cd parity
cargo build --release
```

