# @parity/etherscan

A thin, lightweight promise-based wrapper for the [`api.etherscan.io` service](https://etherscan.io/apis), exposing a common endpoint for use in JavaScript applications.

[https://github.com/ethcore/parity/tree/master/js/src/3rdparty/etherscan](https://github.com/ethcore/parity/tree/master/js/src/3rdparty/etherscan)

## installation

```
npm install --save @parity/etherscan
```

## usage

You will need to include [a `fetch` polyfill](https://github.com/matthew-andrews/isomorphic-fetch) if your target platform doesn't support [`fetch`](https://developer.mozilla.org/en-US/docs/Web/API/WindowOrWorkerGlobalScope/fetch) natively.

```js
require('isomorphic-fetch');

const etherscan = require('@parity/etherscan');

// api calls go here
```

## api

### account (exposed on `etherscan.account`)

- `balance(address)`
- `balances(addresses)` (array or addresses)
- `transactions(address, page)` (page offset starts at 0, returns 25)

### stats (exposed on `etherscan.stats`)

- `price()`
- `supply()`
