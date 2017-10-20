# ethapi-js

A thin, fast, low-level Promise-based wrapper around the Ethereum APIs.

[![Build Status](https://travis-ci.org/jacogr/ethapi-js.svg?branch=master)](https://travis-ci.org/jacogr/ethapi-js)
[![Coverage Status](https://coveralls.io/repos/github/jacogr/ethapi-js/badge.svg?branch=master)](https://coveralls.io/github/jacogr/ethapi-js?branch=master)
[![Dependency Status](https://david-dm.org/jacogr/ethapi-js.svg)](https://david-dm.org/jacogr/ethapi-js)
[![devDependency Status](https://david-dm.org/jacogr/ethapi-js/dev-status.svg)](https://david-dm.org/jacogr/ethapi-js#info=devDependencies)

## contributing

Clone the repo and install dependencies via `npm install`. Tests can be executed via

- `npm run testOnce` (100% covered unit tests)
- `npm run testE2E` (E2E against a running RPC-enabled testnet Parity/Geth instance, `parity --testnet` and for WebScokets, `geth --testnet --ws --wsorigins '*' --rpc`)
- setting the environment `DEBUG=true` will display the RPC POST bodies and responses on E2E tests

## installation

Install the package with `npm install --save ethapi-js` from the [npm registry ethapi-js](https://www.npmjs.com/package/ethapi-js)

## usage

### initialisation

```javascript
// import the actual EthApi class
import EthApi from 'ethapi-js';

// do the setup
const transport = new EthApi.Transport.Http('http://localhost:8545');  // or .Ws('ws://localhost:8546')
const ethapi = new EthApi(transport);
```

You will require native Promises and fetch support (latest browsers only), they can be utilised by

```javascript
import 'isomorphic-fetch';

import es6Promise from 'es6-promise';
es6Promise.polyfill();
```

### making calls

perform a call

```javascript
ethapi.eth
  .coinbase()
  .then((coinbase) => {
    console.log(`The coinbase is ${coinbase}`);
  });
```

multiple promises

```javascript
Promise
  .all([
    ethapi.eth.coinbase(),
    ethapi.net.listening()
  ])
  .then(([coinbase, listening]) => {
    // do stuff here
  });
```

chaining promises

```javascript
ethapi.eth
  .newFilter({...})
  .then((filterId) => ethapi.eth.getFilterChanges(filterId))
  .then((changes) => {
    console.log(changes);
  });
```

### contracts

attach contract

```javascript
const abi = [{ name: 'callMe', inputs: [{ type: 'bool', ...}, { type: 'string', ...}]}, ...abi...];
const contract = new ethapi.newContract(abi);
```

deploy

```javascript
contract
  .deploy('0xc0de', [params], 'superPassword')
  .then((address) => {
    console.log(`the contract was deployed at ${address}`);
  });
```

attach a contract at address

```javascript
// via the constructor & .at function
const contract = api.newContract(abi).at('0xa9280...7347b');
// or on an already initialised contract
contract.at('0xa9280...7347b');
// perform calls here
```

find & call a function

```javascript
contract.instance
  .myContractMethodName
  .call({}, [myContractMethodParameter]) // or estimateGas or sendTransaction
  .then((result) => {
    console.log(`the result was ${result}`);
  });
```

parse events from transaction receipt

```javascript
contract
  .parseTransactionEvents(txReceipt)
  .then((receipt) => {
    receipt.logs.forEach((log) => {
      console.log('log parameters', log.params);
    });
  });
```

## apis

APIs implement the calls as exposed in the [Ethcore JSON Ethereum RPC](https://github.com/paritytech/ethereum-rpc-json/) definitions. Mapping follows the naming conventions of the originals, i.e. `eth_call` becomes `eth.call`, `personal_accounts` becomes `personal.accounts`, etc.

- [ethapi.db](https://github.com/paritytech/ethereum-rpc-json/blob/master/interfaces.md#db)
- [ethapi.eth](https://github.com/paritytech/ethereum-rpc-json/blob/master/interfaces.md#eth)
- [ethapi.parity](https://github.com/paritytech/ethereum-rpc-json/blob/master/interfaces.md#parity)
- [ethapi.net](https://github.com/paritytech/ethereum-rpc-json/blob/master/interfaces.md#net)
- [ethapi.personal](https://github.com/paritytech/ethereum-rpc-json/blob/master/interfaces.md#personal)
- [ethapi.shh](https://github.com/paritytech/ethereum-rpc-json/blob/master/interfaces.md#shh)
- [ethapi.signer](https://github.com/paritytech/ethereum-rpc-json/blob/master/interfaces.md#signer)
- [ethapi.trace](https://github.com/paritytech/ethereum-rpc-json/blob/master/interfaces.md#trace)
- [ethapi.web3](https://github.com/paritytech/ethereum-rpc-json/blob/master/interfaces.md#web3)

As a verification step, all exposed interfaces are tested for existing and pointing to the correct endpoints by using the generated interfaces from the above repo.
