# @parity/ShapeShift

A thin ES6 promise wrapper around the shapeshift.io APIs as documented at https://shapeshift.io/api

[https://github.com/ethcore/parity/js/src/3rdparty/shapeshift](https://github.com/ethcore/parity/js/src/3rdparty/shapeshift)

## usage

installation -

```
npm install --save @parity/ShapeShift
```

Usage -

```
const APIKEY = 'private affiliate key or undefined';
const shapeshift = require('@parity/ShapeShift')(APIKEY);

// api calls goes here
```

## api

queries -

- `getCoins()` [https://shapeshift.io/api#api-104](https://shapeshift.io/api#api-104)
- `getMarketInfo(pair)` [https://shapeshift.io/api#api-103](https://shapeshift.io/api#api-103)
- `getStatus(depositAddress)` [https://shapeshift.io/api#api-5](https://shapeshift.io/api#api-5)

transactions -

- `shift(toAddress, returnAddress, pair)` [https://shapeshift.io/api#api-7](https://shapeshift.io/api#api-7)
