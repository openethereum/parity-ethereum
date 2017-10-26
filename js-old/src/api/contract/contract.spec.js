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

import BigNumber from 'bignumber.js';
import sinon from 'sinon';

import { TEST_HTTP_URL, mockHttp } from '../../../test/mockRpc';

import Abi from '../../abi';
import { sha3 } from '../util/sha3';

import Api from '../api';
import Contract from './contract';
import { isInstanceOf, isFunction } from '../util/types';

const transport = new Api.Transport.Http(TEST_HTTP_URL, -1);
const eth = new Api(transport);

describe('api/contract/Contract', () => {
  const ADDR = '0x0123456789';

  const ABI = [
    {
      type: 'function', name: 'test',
      inputs: [{ name: 'boolin', type: 'bool' }, { name: 'stringin', type: 'string' }],
      outputs: [{ type: 'uint' }]
    },
    {
      type: 'function', name: 'test2',
      outputs: [{ type: 'uint' }, { type: 'uint' }]
    },
    {
      type: 'constructor',
      inputs: [{ name: 'boolin', type: 'bool' }, { name: 'stringin', type: 'string' }]
    },
    { type: 'event', name: 'baz' },
    { type: 'event', name: 'foo' }
  ];

  const ABI_NO_PARAMS = [
    {
      type: 'function', name: 'test',
      inputs: [{ name: 'boolin', type: 'bool' }, { name: 'stringin', type: 'string' }],
      outputs: [{ type: 'uint' }]
    },
    {
      type: 'function', name: 'test2',
      outputs: [{ type: 'uint' }, { type: 'uint' }]
    },
    {
      type: 'constructor'
    },
    { type: 'event', name: 'baz' },
    { type: 'event', name: 'foo' }
  ];

  const VALUES = [ true, 'jacogr' ];
  const CALLDATA = `
    0000000000000000000000000000000000000000000000000000000000000001
    0000000000000000000000000000000000000000000000000000000000000040
    0000000000000000000000000000000000000000000000000000000000000006
    6a61636f67720000000000000000000000000000000000000000000000000000
  `.replace(/\s/g, '');
  const SIGNATURE = '02356205';

  const ENCODED = `0x${SIGNATURE}${CALLDATA}`;

  const RETURN1 = '0000000000000000000000000000000000000000000000000000000000123456';
  const RETURN2 = '0000000000000000000000000000000000000000000000000000000000456789';
  let scope;

  describe('constructor', () => {
    it('needs an EthAbi instance', () => {
      expect(() => new Contract()).to.throw(/API instance needs to be provided to Contract/);
    });

    it('needs an ABI', () => {
      expect(() => new Contract(eth)).to.throw(/ABI needs to be provided to Contract instance/);
    });

    describe('internal setup', () => {
      const contract = new Contract(eth, ABI);

      it('sets EthApi & parsed interface', () => {
        expect(contract.address).to.not.be.ok;
        expect(contract.api).to.deep.equal(eth);
        expect(isInstanceOf(contract.abi, Abi)).to.be.ok;
      });

      it('attaches functions', () => {
        expect(contract.functions.length).to.equal(2);
        expect(contract.functions[0].name).to.equal('test');
      });

      it('attaches constructors', () => {
        expect(contract.constructors.length).to.equal(1);
      });

      it('attaches events', () => {
        expect(contract.events.length).to.equal(2);
        expect(contract.events[0].name).to.equal('baz');
      });
    });
  });

  describe('at', () => {
    it('sets returns the functions, events & sets the address', () => {
      const contract = new Contract(eth, [
        {
          constant: true,
          inputs: [{
            name: '_who',
            type: 'address'
          }],
          name: 'balanceOf',
          outputs: [{
            name: '',
            type: 'uint256'
          }],
          type: 'function'
        },
        {
          anonymous: false,
          inputs: [{
            indexed: false,
            name: 'amount',
            type: 'uint256'
          }],
          name: 'Drained',
          type: 'event'
        }
      ]);

      contract.at('6789');

      expect(Object.keys(contract.instance)).to.deep.equal([
        'Drained',
        /^(?:0x)(.+)$/.exec(sha3('Drained(uint256)'))[1],
        'balanceOf',
        /^(?:0x)(.+)$/.exec(sha3('balanceOf(address)'))[1].substr(0, 8),
        'address'
      ]);
      expect(contract.address).to.equal('6789');
    });
  });

  describe('parseTransactionEvents', () => {
    it('parses a transaction log into the data', () => {
      const contract = new Contract(eth, [
        {
          anonymous: false, name: 'Message', type: 'event',
          inputs: [
            { indexed: true, name: 'postId', type: 'uint256' },
            { indexed: false, name: 'parentId', type: 'uint256' },
            { indexed: false, name: 'sender', type: 'address' },
            { indexed: false, name: 'at', type: 'uint256' },
            { indexed: false, name: 'messageId', type: 'uint256' },
            { indexed: false, name: 'message', type: 'string' }
          ]
        }
      ]);
      const decoded = contract.parseTransactionEvents({
        blockHash: '0xa9280530a3b47bee2fc80f2862fd56502ae075350571d724d6442ea4c597347b',
        blockNumber: '0x4fcd',
        cumulativeGasUsed: '0xb57f',
        gasUsed: '0xb57f',
        logs: [{
          address: '0x22bff18ec62281850546a664bb63a5c06ac5f76c',
          blockHash: '0xa9280530a3b47bee2fc80f2862fd56502ae075350571d724d6442ea4c597347b',
          blockNumber: '0x4fcd',
          data: '0x000000000000000000000000000000000000000000000000000000000000000000000000000000000000000063cf90d3f0410092fc0fca41846f5962239791950000000000000000000000000000000000000000000000000000000056e6c85f0000000000000000000000000000000000000000000000000001000000004fcd00000000000000000000000000000000000000000000000000000000000000a0000000000000000000000000000000000000000000000000000000000000000d706f7374286d6573736167652900000000000000000000000000000000000000',
          logIndex: '0x0',
          topics: [
            '0x954ba6c157daf8a26539574ffa64203c044691aa57251af95f4b48d85ec00dd5',
            '0x0000000000000000000000000000000000000000000000000001000000004fe0'
          ],
          transactionHash: '0xca16f537d761d13e4e80953b754e2b15541f267d6cad9381f750af1bae1e4917',
          transactionIndex: '0x0'
        }],
        to: '0x22bff18ec62281850546a664bb63a5c06ac5f76c',
        transactionHash: '0xca16f537d761d13e4e80953b754e2b15541f267d6cad9381f750af1bae1e4917',
        transactionIndex: '0x0'
      });
      const log = decoded.logs[0];

      expect(log.event).to.equal('Message');
      expect(log.address).to.equal('0x22bff18ec62281850546a664bb63a5c06ac5f76c');
      expect(log.params).to.deep.equal({
        at: { type: 'uint', value: new BigNumber('1457965151') },
        message: { type: 'string', value: 'post(message)' },
        messageId: { type: 'uint', value: new BigNumber('281474976731085') },
        parentId: { type: 'uint', value: new BigNumber(0) },
        postId: { type: 'uint', value: new BigNumber('281474976731104') },
        sender: { type: 'address', value: '0x63Cf90D3f0410092FC0fca41846f596223979195' }
      });
    });
  });

  describe('_pollTransactionReceipt', () => {
    const contract = new Contract(eth, ABI);
    const ADDRESS = '0xD337e80eEdBdf86eDBba021797d7e4e00Bb78351';
    const BLOCKNUMBER = '555000';
    const RECEIPT = { contractAddress: ADDRESS.toLowerCase(), blockNumber: BLOCKNUMBER };
    const EXPECT = { contractAddress: ADDRESS, blockNumber: new BigNumber(BLOCKNUMBER) };

    let scope;
    let receipt;

    describe('success', () => {
      before(() => {
        scope = mockHttp([
          { method: 'eth_getTransactionReceipt', reply: { result: null } },
          { method: 'eth_getTransactionReceipt', reply: { result: null } },
          { method: 'eth_getTransactionReceipt', reply: { result: RECEIPT } }
        ]);

        return contract
          ._pollTransactionReceipt('0x123')
          .then((_receipt) => {
            receipt = _receipt;
          });
      });

      it('sends multiple getTransactionReceipt calls', () => {
        expect(scope.isDone()).to.be.true;
      });

      it('passes the txhash through', () => {
        expect(scope.body.eth_getTransactionReceipt.params[0]).to.equal('0x123');
      });

      it('receives the final receipt', () => {
        expect(receipt).to.deep.equal(EXPECT);
      });
    });

    describe('error', () => {
      before(() => {
        scope = mockHttp([{ method: 'eth_getTransactionReceipt', reply: { error: { code: -1, message: 'failure' } } }]);
      });

      it('returns the errors', () => {
        return contract
          ._pollTransactionReceipt('0x123')
          .catch((error) => {
            expect(error.message).to.match(/failure/);
          });
      });
    });
  });

  describe('deploy without parameters', () => {
    const contract = new Contract(eth, ABI_NO_PARAMS);
    const CODE = '0x123';
    const ADDRESS = '0xD337e80eEdBdf86eDBba021797d7e4e00Bb78351';
    const RECEIPT_DONE = { contractAddress: ADDRESS.toLowerCase(), gasUsed: 50, blockNumber: 2500 };

    let scope;

    describe('success', () => {
      before(() => {
        scope = mockHttp([
          { method: 'eth_estimateGas', reply: { result: 1000 } },
          { method: 'parity_postTransaction', reply: { result: '0x678' } },
          { method: 'parity_checkRequest', reply: { result: '0x890' } },
          { method: 'eth_getTransactionReceipt', reply: { result: RECEIPT_DONE } },
          { method: 'eth_getCode', reply: { result: CODE } }
        ]);

        return contract.deploy({ data: CODE }, []);
      });

      it('passes the options through to postTransaction (incl. gas calculation)', () => {
        expect(scope.body.parity_postTransaction.params[0].data).to.equal(CODE);
      });
    });
  });

  describe('deploy', () => {
    const contract = new Contract(eth, ABI);
    const ADDRESS = '0xD337e80eEdBdf86eDBba021797d7e4e00Bb78351';
    const RECEIPT_PEND = { contractAddress: ADDRESS.toLowerCase(), gasUsed: 50, blockNumber: 0 };
    const RECEIPT_DONE = { contractAddress: ADDRESS.toLowerCase(), gasUsed: 50, blockNumber: 2500 };
    const RECEIPT_EXCP = { contractAddress: ADDRESS.toLowerCase(), gasUsed: 1200, blockNumber: 2500 };

    let scope;

    describe('success', () => {
      before(() => {
        scope = mockHttp([
          { method: 'eth_estimateGas', reply: { result: 1000 } },
          { method: 'parity_postTransaction', reply: { result: '0x678' } },
          { method: 'parity_checkRequest', reply: { result: null } },
          { method: 'parity_checkRequest', reply: { result: '0x890' } },
          { method: 'eth_getTransactionReceipt', reply: { result: null } },
          { method: 'eth_getTransactionReceipt', reply: { result: RECEIPT_PEND } },
          { method: 'eth_getTransactionReceipt', reply: { result: RECEIPT_DONE } },
          { method: 'eth_getCode', reply: { result: '0x456' } }
        ]);

        return contract.deploy({ data: '0x123' }, VALUES);
      });

      it('calls estimateGas, postTransaction, checkRequest, getTransactionReceipt & getCode in order', () => {
        expect(scope.isDone()).to.be.true;
      });

      it('passes the options through to postTransaction (incl. gas calculation)', () => {
        expect(scope.body.parity_postTransaction.params).to.deep.equal([
          { data: `0x123${CALLDATA}`, gas: '0x4b0' }
        ]);
      });

      it('sets the address of the contract', () => {
        expect(contract.address).to.equal(ADDRESS);
      });
    });

    describe('error', () => {
      it('fails when gasUsed == gas', () => {
        mockHttp([
          { method: 'eth_estimateGas', reply: { result: 1000 } },
          { method: 'parity_postTransaction', reply: { result: '0x678' } },
          { method: 'parity_checkRequest', reply: { result: '0x789' } },
          { method: 'eth_getTransactionReceipt', reply: { result: RECEIPT_EXCP } }
        ]);

        return contract
          .deploy({ data: '0x123' }, VALUES)
          .catch((error) => {
            expect(error.message).to.match(/not deployed, gasUsed/);
          });
      });

      it('fails when no code was deployed', () => {
        mockHttp([
          { method: 'eth_estimateGas', reply: { result: 1000 } },
          { method: 'parity_postTransaction', reply: { result: '0x678' } },
          { method: 'parity_checkRequest', reply: { result: '0x789' } },
          { method: 'eth_getTransactionReceipt', reply: { result: RECEIPT_DONE } },
          { method: 'eth_getCode', reply: { result: '0x' } }
        ]);

        return contract
          .deploy({ data: '0x123' }, VALUES)
          .catch((error) => {
            expect(error.message).to.match(/not deployed, getCode/);
          });
      });
    });
  });

  describe('bindings', () => {
    let contract;
    let cons;
    let func;

    beforeEach(() => {
      contract = new Contract(eth, ABI);
      contract.at(ADDR);
      cons = contract.constructors[0];
      func = contract.functions.find((fn) => fn.name === 'test');
    });

    describe('_addOptionsTo', () => {
      it('works on no object specified', () => {
        expect(contract._addOptionsTo()).to.deep.equal({ to: ADDR });
      });

      it('uses the contract address when none specified', () => {
        expect(contract._addOptionsTo({ from: 'me' })).to.deep.equal({ to: ADDR, from: 'me' });
      });

      it('overrides the contract address when specified', () => {
        expect(contract._addOptionsTo({ to: 'you', from: 'me' })).to.deep.equal({ to: 'you', from: 'me' });
      });
    });

    describe('attachments', () => {
      it('attaches .call, .postTransaction & .estimateGas to constructors', () => {
        expect(isFunction(cons.call)).to.be.true;
        expect(isFunction(cons.postTransaction)).to.be.true;
        expect(isFunction(cons.estimateGas)).to.be.true;
      });

      it('attaches .call, .postTransaction & .estimateGas to functions', () => {
        expect(isFunction(func.call)).to.be.true;
        expect(isFunction(func.postTransaction)).to.be.true;
        expect(isFunction(func.estimateGas)).to.be.true;
      });

      it('attaches .call only to constant functions', () => {
        func = (new Contract(eth, [{ type: 'function', name: 'test', constant: true }])).functions[0];

        expect(isFunction(func.call)).to.be.true;
        expect(isFunction(func.postTransaction)).to.be.false;
        expect(isFunction(func.estimateGas)).to.be.false;
      });
    });

    describe('postTransaction', () => {
      beforeEach(() => {
        scope = mockHttp([{ method: 'parity_postTransaction', reply: { result: ['hashId'] } }]);
      });

      it('encodes options and mades an parity_postTransaction call', () => {
        return func
          .postTransaction({ someExtras: 'foo' }, VALUES)
          .then(() => {
            expect(scope.isDone()).to.be.true;
            expect(scope.body.parity_postTransaction.params[0]).to.deep.equal({
              someExtras: 'foo',
              to: ADDR,
              data: ENCODED
            });
          });
      });
    });

    describe('estimateGas', () => {
      beforeEach(() => {
        scope = mockHttp([{ method: 'eth_estimateGas', reply: { result: ['0x123'] } }]);
      });

      it('encodes options and mades an eth_estimateGas call', () => {
        return func
          .estimateGas({ someExtras: 'foo' }, VALUES)
          .then((amount) => {
            expect(scope.isDone()).to.be.true;
            expect(amount.toString(16)).to.equal('123');
            expect(scope.body.eth_estimateGas.params).to.deep.equal([{
              someExtras: 'foo',
              to: ADDR,
              data: ENCODED
            }]);
          });
      });
    });

    describe('call', () => {
      it('encodes options and mades an eth_call call', () => {
        scope = mockHttp([{ method: 'eth_call', reply: { result: RETURN1 } }]);

        return func
          .call({ someExtras: 'foo' }, VALUES)
          .then((result) => {
            expect(scope.isDone()).to.be.true;
            expect(scope.body.eth_call.params).to.deep.equal([{
              someExtras: 'foo',
              to: ADDR,
              data: ENCODED
            }, 'latest']);
            expect(result.toString(16)).to.equal('123456');
          });
      });

      it('encodes options and mades an eth_call call (multiple returns)', () => {
        scope = mockHttp([{ method: 'eth_call', reply: { result: `${RETURN1}${RETURN2}` } }]);

        return contract.functions[1]
          .call({}, [])
          .then((result) => {
            expect(scope.isDone()).to.be.true;
            expect(result.length).to.equal(2);
            expect(result[0].toString(16)).to.equal('123456');
            expect(result[1].toString(16)).to.equal('456789');
          });
      });
    });
  });

  describe('subscribe', () => {
    const abi = [
      {
        anonymous: false, name: 'Message', type: 'event',
        inputs: [
          { indexed: true, name: 'postId', type: 'uint256' },
          { indexed: false, name: 'parentId', type: 'uint256' },
          { indexed: false, name: 'sender', type: 'address' },
          { indexed: false, name: 'at', type: 'uint256' },
          { indexed: false, name: 'messageId', type: 'uint256' },
          { indexed: false, name: 'message', type: 'string' }
        ]
      }
    ];

    const logs = [{
      address: '0x22bff18ec62281850546a664bb63a5c06ac5f76c',
      blockHash: '0xa9280530a3b47bee2fc80f2862fd56502ae075350571d724d6442ea4c597347b',
      blockNumber: '0x4fcd',
      data: '0x000000000000000000000000000000000000000000000000000000000000000000000000000000000000000063cf90d3f0410092fc0fca41846f5962239791950000000000000000000000000000000000000000000000000000000056e6c85f0000000000000000000000000000000000000000000000000001000000004fcd00000000000000000000000000000000000000000000000000000000000000a0000000000000000000000000000000000000000000000000000000000000000d706f7374286d6573736167652900000000000000000000000000000000000000',
      logIndex: '0x0',
      topics: [
        '0x954ba6c157daf8a26539574ffa64203c044691aa57251af95f4b48d85ec00dd5',
        '0x0000000000000000000000000000000000000000000000000001000000004fe0'
      ],
      transactionHash: '0xca16f537d761d13e4e80953b754e2b15541f267d6cad9381f750af1bae1e4917',
      transactionIndex: '0x0'
    }];

    const parsed = [{
      address: '0x22bfF18ec62281850546a664bb63a5C06AC5F76C',
      blockHash: '0xa9280530a3b47bee2fc80f2862fd56502ae075350571d724d6442ea4c597347b',
      blockNumber: new BigNumber(20429),
      data: '0x000000000000000000000000000000000000000000000000000000000000000000000000000000000000000063cf90d3f0410092fc0fca41846f5962239791950000000000000000000000000000000000000000000000000000000056e6c85f0000000000000000000000000000000000000000000000000001000000004fcd00000000000000000000000000000000000000000000000000000000000000a0000000000000000000000000000000000000000000000000000000000000000d706f7374286d6573736167652900000000000000000000000000000000000000',
      event: 'Message',
      logIndex: new BigNumber(0),
      params: {
        at: { type: 'uint', value: new BigNumber(1457965151) },
        message: { type: 'string', value: 'post(message)' },
        messageId: { type: 'uint', value: new BigNumber(281474976731085) },
        parentId: { type: 'uint', value: new BigNumber(0) },
        postId: { type: 'uint', value: new BigNumber(281474976731104) },
        sender: { type: 'address', value: '0x63Cf90D3f0410092FC0fca41846f596223979195' }
      },
      topics: [
        '0x954ba6c157daf8a26539574ffa64203c044691aa57251af95f4b48d85ec00dd5',
        '0x0000000000000000000000000000000000000000000000000001000000004fe0'
      ],
      transactionHash: '0xca16f537d761d13e4e80953b754e2b15541f267d6cad9381f750af1bae1e4917',
      transactionIndex: new BigNumber(0)
    }];

    let contract;

    beforeEach(() => {
      contract = new Contract(eth, abi);
      contract.at(ADDR);
    });

    describe('invalid events', () => {
      it('fails to subscribe to an invalid names', () => {
        return contract
          .subscribe('invalid')
          .catch((error) => {
            expect(error.message).to.match(/invalid is not a valid eventName/);
          });
      });
    });

    describe('valid events', () => {
      let cbb;
      let cbe;

      beforeEach(() => {
        scope = mockHttp([
          { method: 'eth_newFilter', reply: { result: '0x123' } },
          { method: 'eth_getFilterLogs', reply: { result: logs } },
          { method: 'eth_getFilterChanges', reply: { result: logs } },
          { method: 'eth_newFilter', reply: { result: '0x123' } },
          { method: 'eth_getFilterLogs', reply: { result: logs } }
        ]);
        cbb = sinon.stub();
        cbe = sinon.stub();

        return contract.subscribe('Message', { toBlock: 'pending' }, cbb);
      });

      it('sets the subscriptionId returned', () => {
        return contract
          .subscribe('Message', { toBlock: 'pending' }, cbe)
          .then((subscriptionId) => {
            expect(subscriptionId).to.equal(1);
          });
      });

      it('creates a new filter and retrieves the logs on it', () => {
        return contract
          .subscribe('Message', { toBlock: 'pending' }, cbe)
          .then((subscriptionId) => {
            expect(scope.isDone()).to.be.true;
          });
      });

      it('returns the logs to the callback', () => {
        return contract
          .subscribe('Message', { toBlock: 'pending' }, cbe)
          .then((subscriptionId) => {
            expect(cbe).to.have.been.calledWith(null, parsed);
          });
      });
    });
  });
});
