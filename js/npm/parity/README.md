# @parity/parity.js

Parity.js is a thin, fast, Promise-based wrapper around the Ethereum APIs.

[https://github.com/ethcore/parity/tree/master/js/src/api](https://github.com/ethcore/parity/tree/master/js/src/api)

## installation

Install the package with `npm install --save @parity/parity.js`

## usage

### initialisation

```javascript
// import the actual Api class
import { Api } from '@parity/parity.js';

// do the setup
const transport = new Api.Transport.Http('http://localhost:8545');
const api = new Api(transport);
```

### making calls

perform a call

```javascript
api.eth
  .coinbase()
  .then((coinbase) => {
    console.log(`The coinbase is ${coinbase}`);
  });
```

multiple promises

```javascript
Promise
  .all([
    api.eth.coinbase(),
    api.net.listening()
  ])
  .then(([coinbase, listening]) => {
    // do stuff here
  });
```

chaining promises

```javascript
api.eth
  .newFilter({...})
  .then((filterId) => api.eth.getFilterChanges(filterId))
  .then((changes) => {
    console.log(changes);
  });
```

### contracts

attach contract

```javascript
const abi = [{ name: 'callMe', inputs: [{ type: 'bool', ...}, { type: 'string', ...}]}, ...abi...];
const address = '0x123456...9abc';
const contract = new api.newContract(abi, address);
```

find & call a function

```javascript
contract.instance
  .callMe
  .call({ gas: 21000 }, [true, 'someString']) // or estimateGas or postTransaction
  .then((result) => {
    console.log(`the result was ${result}`);
  });
```

## apis

APIs implement the calls as exposed in the [Ethcore JSON Ethereum RPC](https://github.com/ethcore/ethereum-rpc-json/) definitions. Mapping follows the naming conventions of the originals, i.e. `eth_call` becomes `eth.call`, `personal_accounts` becomes `personal.accounts`, etc.
