// Copyright 2015-2017 Parity Technologies (UK) Ltd.
// This file is part of Parity.

// Parity is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Parity is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Parity.  If not, see <http://www.gnu.org/licenses/>.

import Web3 from 'web3';

const api = window.parent.secureApi;
let web3;

api.parity.dappsUrl().then(url => {
  web3 = new Web3(new Web3.providers.HttpProvider(`${window.location.protocol}//${url}/rpc/`));
  window.web3 = web3;

  // Usage example:
  // web3.eth.traceCall({
  //     to: theChicken.address,
  //     data: theChicken.withdraw.getData(100000000000000000),
  //     gas: 100000
  //   },
  //   `["trace", "vmTrace", "stateDiff"]
  //  )
  web3._extend({
    property: 'eth',
    methods: [
      new web3._extend.Method({
        name: 'traceCall',
        call: 'trace_call',
        params: 2,
        inputFormatter: [web3._extend.formatters.inputCallFormatter, null]
      })
    ]
  });

  web3._extend({
    property: 'eth',
    methods: [
      new web3._extend.Method({
        name: 'traceSendRawTransaction',
        call: 'trace_rawTransaction',
        params: 2,
        inputFormatter: [null, null]
      })
    ]
  });

  web3._extend({
    property: 'eth',
    methods: [
      new web3._extend.Method({
        name: 'traceReplayTransaction',
        call: 'trace_replayTransaction',
        params: 2,
        inputFormatter: [null, null]
      })
    ]
  });

  web3._extend({
    property: 'eth',
    methods: [
      new web3._extend.Method({
        name: 'setMode',
        call: 'parity_setMode',
        params: 1
      })
    ]
  });

  web3._extend({
    property: 'eth',
    methods: [
      new web3._extend.Method({
        name: 'mode',
        call: 'parity_mode',
        params: 0
      })
    ]
  });

  web3._extend({
    property: 'eth',
    methods: [
      new web3._extend.Method({
        name: 'traceTransaction',
        call: 'trace_Transaction',
        params: 1,
        inputFormatter: [null]
      })
    ]
  });

  web3._extend({
    property: 'eth',
    methods: [
      new web3._extend.Method({
        name: 'gasPriceStatistics',
        call: 'parity_gasPriceStatistics',
        params: 0,
        outputFormatter: function (a) { return a.map(web3.toBigNumber); }
      })
    ]
  });

  web3._extend({
    property: 'eth',
    methods: [
      new web3._extend.Method({
        name: 'registryAddress',
        call: 'parity_registryAddress',
        params: 0
      })
    ]
  });

  web3._extend({
    property: 'eth',
    methods: [
      new web3._extend.Method({
        name: 'accountsInfo',
        call: 'personal_accountsInfo',
        outputFormatter: function (m) {
          Object.keys(m).forEach(k => {
            m[k].meta = JSON.parse(m[k].meta);
            m[k].meta.name = m[k].name;
            m[k].meta.uuid = m[k].uuid;
            m[k] = m[k].meta;
          }); return m;
        },
        params: 0
      })
    ]
  });

  web3._extend({
    property: 'eth',
    methods: [
      new web3._extend.Method({
        name: 'setAccountName',
        call: 'personal_setAccountName',
        params: 2
      })
    ]
  });

  web3._extend({
    property: 'eth',
    methods: [
      new web3._extend.Method({
        name: 'setAccountMeta',
        call: 'personal_setAccountMeta',
        params: 2,
        inputFormatter: [a => a, JSON.stringify]
      })
    ]
  });

  web3._extend({
    property: 'eth',
    methods: [
      new web3._extend.Method({
        name: 'postTransaction',
        call: 'eth_postTransaction',
        params: 1,
        inputFormatter: [web3._extend.formatters.inputCallFormatter]
      })
    ]
  });

  web3._extend({
    property: 'eth',
    methods: [
      new web3._extend.Method({
        name: 'postSign',
        call: 'eth_postSign',
        params: 1
      })
    ]
  });

  web3._extend({
    property: 'eth',
    methods: [
      new web3._extend.Method({
        name: 'encryptMessage',
        call: 'parity_encryptMessage',
        params: 2
      })
    ]
  });

  web3._extend({
    property: 'eth',
    methods: [
      new web3._extend.Method({
        name: 'checkRequest',
        call: 'eth_checkRequest',
        params: 1
      })
    ]
  });

  web3._extend({
    property: 'eth',
    methods: [
      new web3._extend.Method({
        name: 'listAccounts',
        call: 'parity_listAccounts',
        params: 0
      })
    ]
  });

  {
    let postTransaction = web3.eth.postTransaction.bind(web3.eth);
    let sendTransaction = web3.eth.sendTransaction.bind(web3.eth);

    web3.eth.sendTransaction = function (options, f) {
      // No callback - do sync API.
      if (typeof f !== 'function') {
        return sendTransaction(options);
      }
      // Callback - use async API.
      let id = postTransaction(options);

      console.log('Posted trasaction id=' + id);
      let timerId = window.setInterval(check, 500);

      function check () {
        try {
          let r = web3.eth.checkRequest(id);

          if (typeof r === 'string') {
            clearInterval(timerId);
            if (r === '0x0000000000000000000000000000000000000000000000000000000000000000') {
              f('Rejected', r);
            } else {
              f(null, r);
            }
          } else if (r !== null) {
            console.log('checkRequest returned: ' + r);
          }
        } catch (e) {
          clearInterval(timerId);
          f('Rejected', null);
        }
      }
    };
  }

  web3.eth.installInterceptor = function (interceptor) {
    let oldSendTransaction = web3.eth.sendTransaction.bind(web3.eth);

    web3.eth.sendTransaction = function (options, f) {
      if (!interceptor(options)) {
        return '0x0000000000000000000000000000000000000000000000000000000000000000';
      }

      return oldSendTransaction(options, f);
    };
  };

  web3.eth.reporter = function (e, r) {
    if (e) {
      console.log('Error confirming transaction: ' + e);
    } else {
      let addr = r;
      let confirmed = false;
      let timerId = window.setInterval(function check () {
        let receipt = web3.eth.getTransactionReceipt(addr);

        if (receipt != null) {
          if (!confirmed) {
            console.log('Transaction confirmed (' + r + '); used ' + receipt.gasUsed + ' gas; left ' + receipt.logs.length + ' logs; mining...');
            confirmed = true;
          }
          if (typeof receipt.blockHash === 'string') {
            clearInterval(timerId);
            console.log('Mined into block ' + receipt.blockNumber);
          }
        }
      }, 500);
    }
  };

  {
    let oldSha3 = web3.sha3;

    web3.sha3 = function (data, format) {
      if (typeof format !== 'string' || (format !== 'hex' && format !== 'bin')) {
        format = data.startsWith('0x') ? 'hex' : 'bin';
      }
      return oldSha3(data, { encoding: format });
    };
  }

  {
    let Registry = web3.eth.contract([{ 'constant': false, 'inputs': [{ 'name': '_new', 'type': 'address' }], 'name': 'setOwner', 'outputs': [], 'type': 'function' }, { 'constant': false, 'inputs': [{ 'name': '_name', 'type': 'string' }], 'name': 'confirmReverse', 'outputs': [{ 'name': 'success', 'type': 'bool' }], 'type': 'function' }, { 'constant': false, 'inputs': [{ 'name': '_name', 'type': 'bytes32' }], 'name': 'reserve', 'outputs': [{ 'name': 'success', 'type': 'bool' }], 'type': 'function' }, { 'constant': false, 'inputs': [{ 'name': '_name', 'type': 'bytes32' }, { 'name': '_key', 'type': 'string' }, { 'name': '_value', 'type': 'bytes32' }], 'name': 'set', 'outputs': [{ 'name': 'success', 'type': 'bool' }], 'type': 'function' }, { 'constant': false, 'inputs': [{ 'name': '_name', 'type': 'bytes32' }], 'name': 'drop', 'outputs': [{ 'name': 'success', 'type': 'bool' }], 'type': 'function' }, { 'constant': true, 'inputs': [{ 'name': '_name', 'type': 'bytes32' }, { 'name': '_key', 'type': 'string' }], 'name': 'getAddress', 'outputs': [{ 'name': '', 'type': 'address' }], 'type': 'function' }, { 'constant': false, 'inputs': [{ 'name': '_amount', 'type': 'uint256' }], 'name': 'setFee', 'outputs': [], 'type': 'function' }, { 'constant': false, 'inputs': [{ 'name': '_name', 'type': 'bytes32' }, { 'name': '_to', 'type': 'address' }], 'name': 'transfer', 'outputs': [{ 'name': 'success', 'type': 'bool' }], 'type': 'function' }, { 'constant': true, 'inputs': [], 'name': 'owner', 'outputs': [{ 'name': '', 'type': 'address' }], 'type': 'function' }, { 'constant': true, 'inputs': [{ 'name': '_name', 'type': 'bytes32' }], 'name': 'reserved', 'outputs': [{ 'name': 'reserved', 'type': 'bool' }], 'type': 'function' }, { 'constant': false, 'inputs': [], 'name': 'drain', 'outputs': [], 'type': 'function' }, { 'constant': false, 'inputs': [{ 'name': '_name', 'type': 'string' }, { 'name': '_who', 'type': 'address' }], 'name': 'proposeReverse', 'outputs': [{ 'name': 'success', 'type': 'bool' }], 'type': 'function' }, { 'constant': true, 'inputs': [{ 'name': '_name', 'type': 'bytes32' }, { 'name': '_key', 'type': 'string' }], 'name': 'getUint', 'outputs': [{ 'name': '', 'type': 'uint256' }], 'type': 'function' }, { 'constant': true, 'inputs': [{ 'name': '_name', 'type': 'bytes32' }, { 'name': '_key', 'type': 'string' }], 'name': 'get', 'outputs': [{ 'name': '', 'type': 'bytes32' }], 'type': 'function' }, { 'constant': true, 'inputs': [], 'name': 'fee', 'outputs': [{ 'name': '', 'type': 'uint256' }], 'type': 'function' }, { 'constant': true, 'inputs': [{ 'name': '', 'type': 'address' }], 'name': 'reverse', 'outputs': [{ 'name': '', 'type': 'string' }], 'type': 'function' }, { 'constant': false, 'inputs': [{ 'name': '_name', 'type': 'bytes32' }, { 'name': '_key', 'type': 'string' }, { 'name': '_value', 'type': 'uint256' }], 'name': 'setUint', 'outputs': [{ 'name': 'success', 'type': 'bool' }], 'type': 'function' }, { 'constant': false, 'inputs': [], 'name': 'removeReverse', 'outputs': [], 'type': 'function' }, { 'constant': false, 'inputs': [{ 'name': '_name', 'type': 'bytes32' }, { 'name': '_key', 'type': 'string' }, { 'name': '_value', 'type': 'address' }], 'name': 'setAddress', 'outputs': [{ 'name': 'success', 'type': 'bool' }], 'type': 'function' }, { 'anonymous': false, 'inputs': [{ 'indexed': false, 'name': 'amount', 'type': 'uint256' }], 'name': 'Drained', 'type': 'event' }, { 'anonymous': false, 'inputs': [{ 'indexed': false, 'name': 'amount', 'type': 'uint256' }], 'name': 'FeeChanged', 'type': 'event' }, { 'anonymous': false, 'inputs': [{ 'indexed': true, 'name': 'name', 'type': 'bytes32' }, { 'indexed': true, 'name': 'owner', 'type': 'address' }], 'name': 'Reserved', 'type': 'event' }, { 'anonymous': false, 'inputs': [{ 'indexed': true, 'name': 'name', 'type': 'bytes32' }, { 'indexed': true, 'name': 'oldOwner', 'type': 'address' }, { 'indexed': true, 'name': 'newOwner', 'type': 'address' }], 'name': 'Transferred', 'type': 'event' }, { 'anonymous': false, 'inputs': [{ 'indexed': true, 'name': 'name', 'type': 'bytes32' }, { 'indexed': true, 'name': 'owner', 'type': 'address' }], 'name': 'Dropped', 'type': 'event' }, { 'anonymous': false, 'inputs': [{ 'indexed': true, 'name': 'name', 'type': 'bytes32' }, { 'indexed': true, 'name': 'owner', 'type': 'address' }, { 'indexed': true, 'name': 'key', 'type': 'string' }, { 'indexed': false, 'name': 'plainKey', 'type': 'string' }], 'name': 'DataChanged', 'type': 'event' }, { 'anonymous': false, 'inputs': [{ 'indexed': true, 'name': 'name', 'type': 'string' }, { 'indexed': true, 'name': 'reverse', 'type': 'address' }], 'name': 'ReverseProposed', 'type': 'event' }, { 'anonymous': false, 'inputs': [{ 'indexed': true, 'name': 'name', 'type': 'string' }, { 'indexed': true, 'name': 'reverse', 'type': 'address' }], 'name': 'ReverseConfirmed', 'type': 'event' }, { 'anonymous': false, 'inputs': [{ 'indexed': true, 'name': 'name', 'type': 'string' }, { 'indexed': true, 'name': 'reverse', 'type': 'address' }], 'name': 'ReverseRemoved', 'type': 'event' }, { 'anonymous': false, 'inputs': [{ 'indexed': true, 'name': 'old', 'type': 'address' }, { 'indexed': true, 'name': 'current', 'type': 'address' }], 'name': 'NewOwner', 'type': 'event' }]);

    web3.eth.registry = Registry.at(web3.eth.registryAddress());
    web3.eth.registry.lookup = (name, field) => web3.eth.registry.get(web3.sha3(name), field);
    web3.eth.registry.lookupAddress = (name, field) => web3.eth.registry.getAddress(web3.sha3(name), field);
    web3.eth.registry.lookupUint = (name, field) => web3.eth.registry.getUint(web3.sha3(name), field);

    let TokenReg = web3.eth.contract([{ 'constant': true, 'inputs': [{ 'name': '_id', 'type': 'uint256' }], 'name': 'token', 'outputs': [{ 'name': 'addr', 'type': 'address' }, { 'name': 'tla', 'type': 'string' }, { 'name': 'base', 'type': 'uint256' }, { 'name': 'name', 'type': 'string' }, { 'name': 'owner', 'type': 'address' }], 'type': 'function' }, { 'constant': false, 'inputs': [{ 'name': '_new', 'type': 'address' }], 'name': 'setOwner', 'outputs': [], 'type': 'function' }, { 'constant': false, 'inputs': [{ 'name': '_addr', 'type': 'address' }, { 'name': '_tla', 'type': 'string' }, { 'name': '_base', 'type': 'uint256' }, { 'name': '_name', 'type': 'string' }], 'name': 'register', 'outputs': [{ 'name': '', 'type': 'bool' }], 'type': 'function' }, { 'constant': false, 'inputs': [{ 'name': '_fee', 'type': 'uint256' }], 'name': 'setFee', 'outputs': [], 'type': 'function' }, { 'constant': true, 'inputs': [{ 'name': '_id', 'type': 'uint256' }, { 'name': '_key', 'type': 'bytes32' }], 'name': 'meta', 'outputs': [{ 'name': '', 'type': 'bytes32' }], 'type': 'function' }, { 'constant': false, 'inputs': [{ 'name': '_addr', 'type': 'address' }, { 'name': '_tla', 'type': 'string' }, { 'name': '_base', 'type': 'uint256' }, { 'name': '_name', 'type': 'string' }, { 'name': '_owner', 'type': 'address' }], 'name': 'registerAs', 'outputs': [{ 'name': '', 'type': 'bool' }], 'type': 'function' }, { 'constant': true, 'inputs': [{ 'name': '_tla', 'type': 'string' }], 'name': 'fromTLA', 'outputs': [{ 'name': 'id', 'type': 'uint256' }, { 'name': 'addr', 'type': 'address' }, { 'name': 'base', 'type': 'uint256' }, { 'name': 'name', 'type': 'string' }, { 'name': 'owner', 'type': 'address' }], 'type': 'function' }, { 'constant': true, 'inputs': [], 'name': 'owner', 'outputs': [{ 'name': '', 'type': 'address' }], 'type': 'function' }, { 'constant': false, 'inputs': [], 'name': 'drain', 'outputs': [], 'type': 'function' }, { 'constant': true, 'inputs': [], 'name': 'tokenCount', 'outputs': [{ 'name': '', 'type': 'uint256' }], 'type': 'function' }, { 'constant': false, 'inputs': [{ 'name': '_id', 'type': 'uint256' }], 'name': 'unregister', 'outputs': [], 'type': 'function' }, { 'constant': true, 'inputs': [{ 'name': '_addr', 'type': 'address' }], 'name': 'fromAddress', 'outputs': [{ 'name': 'id', 'type': 'uint256' }, { 'name': 'tla', 'type': 'string' }, { 'name': 'base', 'type': 'uint256' }, { 'name': 'name', 'type': 'string' }, { 'name': 'owner', 'type': 'address' }], 'type': 'function' }, { 'constant': false, 'inputs': [{ 'name': '_id', 'type': 'uint256' }, { 'name': '_key', 'type': 'bytes32' }, { 'name': '_value', 'type': 'bytes32' }], 'name': 'setMeta', 'outputs': [], 'type': 'function' }, { 'constant': true, 'inputs': [], 'name': 'fee', 'outputs': [{ 'name': '', 'type': 'uint256' }], 'type': 'function' }, { 'anonymous': false, 'inputs': [{ 'indexed': true, 'name': 'tla', 'type': 'string' }, { 'indexed': true, 'name': 'id', 'type': 'uint256' }, { 'indexed': false, 'name': 'addr', 'type': 'address' }, { 'indexed': false, 'name': 'name', 'type': 'string' }], 'name': 'Registered', 'type': 'event' }, { 'anonymous': false, 'inputs': [{ 'indexed': true, 'name': 'tla', 'type': 'string' }, { 'indexed': true, 'name': 'id', 'type': 'uint256' }], 'name': 'Unregistered', 'type': 'event' }, { 'anonymous': false, 'inputs': [{ 'indexed': true, 'name': 'id', 'type': 'uint256' }, { 'indexed': true, 'name': 'key', 'type': 'bytes32' }, { 'indexed': false, 'name': 'value', 'type': 'bytes32' }], 'name': 'MetaChanged', 'type': 'event' }, { 'anonymous': false, 'inputs': [{ 'indexed': true, 'name': 'old', 'type': 'address' }, { 'indexed': true, 'name': 'current', 'type': 'address' }], 'name': 'NewOwner', 'type': 'event' }]);

    web3.eth.tokenReg = TokenReg.at(web3.eth.registry.lookupAddress('tokenreg', 'A'));
  }
})
.catch((error) => {
  console.error(error);
});

window.api = api;
window.web3 = web3;

export {
  api,
  web3
};
