# @parity/etherscan

A thin, lightweight promise wrapper for the api.etherscan.io/apis service, exposing a common endpoint for use in JavaScript applications.

[https://github.com/paritytech/parity/tree/master/js/src/3rdparty/etherscan](https://github.com/paritytech/parity/tree/master/js/src/3rdparty/etherscan)

## usage

installation -

```
npm install --save @parity/etherscan
```

Usage -

```
const etherscan = require('@parity/etherscan');

// api calls goes here
```

## api

account (exposed on etherscan.account) -

- `balance(address)`
- `balances(addresses)` (array or addresses)
- `transactions(address, page)` (page offset starts at 0, returns 25)

stats (exposed on etherscan.stats) -

- `price()`
- `supply()`
