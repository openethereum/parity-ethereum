# @parity/shapeshift

A thin promise-based wrapper around [the shapeshift.io APIs](https://shapeshift.io/api).

[https://github.com/ethcore/parity/tree/master/js/src/3rdparty/shapeshift](https://github.com/ethcore/parity/tree/master/js/src/3rdparty/shapeshift)

## usage

```
npm install --save @parity/etherscan
```

## usage

You will need to include [`babel-polyfill`](https://github.com/babel/babel/tree/master/packages/babel-polyfill) as well as [a `Promise` polyfill](https://github.com/stefanpenner/es6-promise#readme) and [a `fetch` polyfill](https://github.com/matthew-andrews/isomorphic-fetch) if your target platform doesn't support them.

```js
require('babel-polyfill');
require('es6-promise').polyfill();
require('isomorphic-fetch');

const APIKEY = 'private affiliate key or undefined';
const shapeshift = require('@parity/shapeshift')(APIKEY);

// api calls go here
```

## api

### queries

- `getCoins()` [https://shapeshift.io/api#api-104](https://shapeshift.io/api#api-104)
- `getMarketInfo(pair)` [https://shapeshift.io/api#api-103](https://shapeshift.io/api#api-103)
- `getStatus(depositAddress)` [https://shapeshift.io/api#api-5](https://shapeshift.io/api#api-5)

### transactions

- `shift(toAddress, returnAddress, pair)` [https://shapeshift.io/api#api-7](https://shapeshift.io/api#api-7)
