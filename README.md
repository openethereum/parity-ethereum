# ethcore

[![Build Status][travis-image]][travis-url] [![Coverage Status][coveralls-image]][coveralls-url] [![Join the chat at https://gitter.im/trogdoro/xiki][gitter-image]][gitter-url] [![GPLv3][license-image]][license-url]

[travis-image]: https://travis-ci.org/ethcore/parity.svg?branch=master
[travis-url]: https://travis-ci.org/ethcore/parity
[coveralls-image]: https://coveralls.io/repos/github/ethcore/parity/badge.svg?branch=master
[coveralls-url]: https://coveralls.io/github/ethcore/parity?branch=master
[gitter-image]: https://badges.gitter.im/Join%20Chat.svg
[gitter-url]: https://gitter.im/ethcore/parity?utm_source=badge&utm_medium=badge&utm_campaign=pr-badge&utm_content=badge
[license-image]: https://img.shields.io/badge/license-GPL%20v3-green.svg
[license-url]: http://www.gnu.org/licenses/gpl-3.0.en.html

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

# install beta
multirust update beta

# download and build parity
git clone https://github.com/ethcore/parity
cd parity

# parity should be build with rust beta
multirust override beta

# build in release
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

# install rust beta
curl -sf https://raw.githubusercontent.com/brson/multirust/master/blastoff.sh | sudo sh -s -- --yes

# install beta
multirust update beta

# download and build parity
git clone https://github.com/ethcore/parity
cd parity

# parity should be build with rust beta
multirust override beta

# build in release
cargo build --release
```

##### OSX with Homebrew

```bash
# install rocksdb && multirust
brew update
brew install rocksdb
brew install multirust

# install beta
multirust update beta

# download and build parity
git clone https://github.com/ethcore/parity
cd parity

# use rust beta for building parity
multirust override beta

cargo build --release
```

