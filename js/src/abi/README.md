# ethabi-js

A very early, very POC-type port of [https://github.com/paritytech/ethabi](https://github.com/paritytech/ethabi) to JavaScript

[![Build Status](https://travis-ci.org/jacogr/ethabi-js.svg?branch=master)](https://travis-ci.org/jacogr/ethabi-js)
[![Coverage Status](https://coveralls.io/repos/github/jacogr/ethabi-js/badge.svg?branch=master)](https://coveralls.io/github/jacogr/ethabi-js?branch=master)
[![Dependency Status](https://david-dm.org/jacogr/ethabi-js.svg)](https://david-dm.org/jacogr/ethabi-js)
[![devDependency Status](https://david-dm.org/jacogr/ethabi-js/dev-status.svg)](https://david-dm.org/jacogr/ethabi-js#info=devDependencies)

## contributing

Clone the repo and install dependencies via `npm install`. Tests can be executed via

- `npm run testOnce` (100% covered unit tests)

## installation

Install the package with `npm install --save ethabi-js` from the [npm registry ethabi-js](https://www.npmjs.com/package/ethabi-js)


## implementation
### approach

- this version tries to stay as close to the original Rust version in intent, function names & purpose
- it is a basic port of the Rust version, relying on effectively the same test-suite (expanded where deemed appropriate)
- it is meant as a library to be used in other projects, i.e. [ethapi-js](https://www.npmjs.com/package/ethapi-js)

### differences to original Rust version

- internally the library operates on string binary representations as opposed to Vector bytes, lengths are therefore 64 bytes as opposed to 32 bytes
- function names are adapted from the Rust standard snake_case to the JavaScript standard camelCase
- due to the initial library focus, the cli component (as implemented by the original) is not supported nor mplemented
