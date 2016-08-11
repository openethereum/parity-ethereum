import BigNumber from 'bignumber.js';

import { TEST_HTTP_URL, mockHttp } from '../../../test/mockRpc';

import Abi from '../../abi';

import Api from '../api';
import Contract from './contract';
import { isInstanceOf, isFunction } from '../util/types';

const transport = new Api.Transport.Http(TEST_HTTP_URL);
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
    { type: 'constructor' },
    { type: 'event', name: 'baz' },
    { type: 'event', name: 'foo' }
  ];
  const VALUES = [true, 'jacogr'];
  const ENCODED = '0x023562050000000000000000000000000000000000000000000000000000000000000001000000000000000000000000000000000000000000000000000000000000004000000000000000000000000000000000000000000000000000000000000000066a61636f67720000000000000000000000000000000000000000000000000000';
  const RETURN1 = '0000000000000000000000000000000000000000000000000000000000123456';
  const RETURN2 = '0000000000000000000000000000000000000000000000000000000000456789';
  let scope;

  describe('constructor', () => {
    it('needs an EthAbi instance', () => {
      expect(() => new Contract()).to.throw(/EthApi needs to be provided/);
    });

    it('needs an ABI', () => {
      expect(() => new Contract(eth)).to.throw(/Object ABI needs/);
    });

    describe('internal setup', () => {
      const contract = new Contract(eth, ABI);

      it('sets EthApi & parsed interface', () => {
        expect(contract.address).to.not.be.ok;
        expect(contract.eth).to.deep.equal(eth);
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
    it('sets returns the instance & sets the address', () => {
      const contract = new Contract(eth, []);

      expect(contract.at('123')).to.deep.equal(contract);
      expect(contract.at('456').address).to.equal('456');
    });
  });

  describe('parseTransactionEvents', () => {
    it('checks for unmatched signatures', () => {
      const contract = new Contract(eth, [{ anonymous: false, name: 'Message', type: 'event' }]);
      expect(() => contract.parseTransactionEvents({
        logs: [{
          data: '0x000000000000000000000000000000000000000000000000000000000000000000000000000000000000000063cf90d3f0410092fc0fca41846f5962239791950000000000000000000000000000000000000000000000000000000056e6c85f0000000000000000000000000000000000000000000000000001000000004fcd00000000000000000000000000000000000000000000000000000000000000a0000000000000000000000000000000000000000000000000000000000000000d706f7374286d6573736167652900000000000000000000000000000000000000',
          topics: [
            '0x954ba6c157daf8a26539574ffa64203c044691aa57251af95f4b48d85ec00dd5',
            '0x0000000000000000000000000000000000000000000000000001000000004fe0'
          ]
        }]
      })).to.throw(/event matching signature/);
    });

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

      expect(decoded.logs[0].params).to.deep.equal({
        at: new BigNumber('1457965151'),
        message: 'post(message)',
        messageId: new BigNumber('281474976731085'),
        parentId: new BigNumber(0),
        postId: new BigNumber('281474976731104'),
        sender: '63cf90d3f0410092fc0fca41846f596223979195'
      });
    });
  });

  describe('pollTransactionReceipt', () => {
    const contract = new Contract(eth, ABI);
    const RECEIPT = { contractAddress: '0xd337e80eedbdf86edbba021797d7e4e00bb78351' };
    const EXPECT = { contractAddress: '0xD337e80eEdBdf86eDBba021797d7e4e00Bb78351' };

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
          .pollTransactionReceipt('0x123')
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
          .pollTransactionReceipt('0x123')
          .catch((error) => {
            expect(error.message).to.match(/failure/);
          });
      });
    });
  });

  describe('deploy', () => {
    const contract = new Contract(eth, ABI);
    const RECEIPT = { contractAddress: '0xd337e80eedbdf86edbba021797d7e4e00bb78351' };
    const ADDRESS = '0xD337e80eEdBdf86eDBba021797d7e4e00Bb78351';

    let scope;

    describe('success', () => {
      before(() => {
        scope = mockHttp([
          { method: 'personal_signAndSendTransaction', reply: { result: '0x678' } },
          { method: 'eth_getTransactionReceipt', reply: { result: null } },
          { method: 'eth_getTransactionReceipt', reply: { result: RECEIPT } },
          { method: 'eth_getCode', reply: { result: '0x456' } }
        ]);

        return contract.deploy('0x123', [], 'xxx');
      });

      it('calls sendTransaction, getTransactionReceipt & getCode in order', () => {
        expect(scope.isDone()).to.be.true;
      });

      it('passes the options & password through to sendTransaction', () => {
        expect(scope.body.personal_signAndSendTransaction.params).to.deep.equal([
          { data: '0x123', gas: '0xdbba0' },
          'xxx'
        ]);
      });

      it('sets the address of the contract', () => {
        expect(contract.address).to.equal(ADDRESS);
      });
    });

    describe('error', () => {
      before(() => {
        scope = mockHttp([
          { method: 'personal_signAndSendTransaction', reply: { result: '0x678' } },
          { method: 'eth_getTransactionReceipt', reply: { result: RECEIPT } },
          { method: 'eth_getCode', reply: { result: '0x' } }
        ]);
      });

      it('fails when no code was deployed', () => {
        return contract
          .deploy('0x123', [], 'xxx')
          .catch((error) => {
            expect(error.message).to.match(/not deployed/);
          });
      });
    });
  });

  describe('bindings', () => {
    let contract;
    let cons;
    let func;

    beforeEach(() => {
      contract = new Contract(eth, ABI).at(ADDR);
      cons = contract.constructors[0];
      func = contract.functions.find((fn) => fn.name === 'test');
    });

    describe('attachments', () => {
      it('attaches .call, .sendTransaction, .signAndSend & .estimateGas to constructors', () => {
        expect(isFunction(cons.call)).to.be.true;
        expect(isFunction(cons.sendTransaction)).to.be.true;
        expect(isFunction(cons.signAndSendTransaction)).to.be.true;
        expect(isFunction(cons.estimateGas)).to.be.true;
      });

      it('attaches .call, .sendTransaction, .signAndSend & .estimateGas to functions', () => {
        expect(isFunction(func.call)).to.be.true;
        expect(isFunction(func.sendTransaction)).to.be.true;
        expect(isFunction(func.signAndSendTransaction)).to.be.true;
        expect(isFunction(func.estimateGas)).to.be.true;
      });

      it('attaches .call only to constant functions', () => {
        func = (new Contract(eth, [{ type: 'function', name: 'test', constant: true }])).functions[0];

        expect(isFunction(func.call)).to.be.true;
        expect(isFunction(func.sendTransaction)).to.be.false;
        expect(isFunction(func.signAndSendTransaction)).to.be.false;
        expect(isFunction(func.estimateGas)).to.be.false;
      });
    });

    describe('sendTransaction', () => {
      beforeEach(() => {
        scope = mockHttp([{ method: 'eth_sendTransaction', reply: { result: ['hashId'] } }]);
      });

      it('encodes options and mades an eth_sendTransaction call', () => {
        return func
          .sendTransaction({ someExtras: 'foo' }, VALUES)
          .then(() => {
            expect(scope.isDone()).to.be.true;
            expect(scope.body.eth_sendTransaction.params[0]).to.deep.equal({
              someExtras: 'foo',
              to: ADDR,
              data: ENCODED
            });
          });
      });
    });

    describe('signAndSendTransaction', () => {
      beforeEach(() => {
        scope = mockHttp([{ method: 'personal_signAndSendTransaction', reply: { result: ['hashId'] } }]);
      });

      it('encodes options and mades an personal_signAndSendTransaction call', () => {
        return func
          .signAndSendTransaction({ someExtras: 'foo' }, VALUES, 'xxx')
          .then(() => {
            expect(scope.isDone()).to.be.true;
            expect(scope.body.personal_signAndSendTransaction.params).to.deep.equal([{
              someExtras: 'foo',
              to: ADDR,
              data: ENCODED
            }, 'xxx']);
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
});
