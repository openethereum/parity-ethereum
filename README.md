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

First (if you don't already have it) get multirust:

- Linux:
```bash
curl -sf https://raw.githubusercontent.com/brson/multirust/master/quick-install.sh | sh
```

- OSX with Homebrew:
```bash
brew update && brew install multirust
multirust default stable
```

Then, download and build Parity:

```bash
# download Parity code
git clone https://github.com/ethcore/parity
cd parity

# build in release mode
cargo build --release
```
