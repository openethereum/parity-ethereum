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
import { TEST_WS_URL, mockWs } from '../../../test/mockRpc';
import { isBigNumber } from '../../../test/types';

import Ws from '../transport/ws';
import Pubsub from './pubsub';

describe('api/pubsub/Pubsub', () => {
  let scope;
  let instance;
  const address = '0x63Cf90D3f0410092FC0fca41846f596223979195';

  describe('accountsInfo', () => {
    beforeEach(() => {
      scope = mockWs([{ method: 'parity_subscribe', reply: 2, subscription: {
        method: 'parity_subscription',
        params: {
          result: {
            '0x63cf90d3f0410092fc0fca41846f596223979195': {
              name: 'name', uuid: 'uuid', meta: '{"data":"data"}'
            }
          },
          subscription: 2
        }
      } }]);
      instance = new Pubsub(new Ws(TEST_WS_URL));
    });

    afterEach(() => {
      scope.stop();
    });

    it('retrieves the available account info', (done) => {
      instance.parity.accountsInfo((error, result) => {
        expect(error).to.be.null;
        expect(result).to.deep.equal({
          '0x63Cf90D3f0410092FC0fca41846f596223979195': {
            name: 'name', uuid: 'uuid', meta: {
              data: 'data'
            }
          }
        });
        done();
      });
    });
  });

  describe('Unsubscribe', () => {
    beforeEach(() => {
      scope = mockWs([{ method: 'parity_subscribe', reply: 2 },
                      { method: 'parity_unsubscribe', reply: true }]);
      instance = new Pubsub(new Ws(TEST_WS_URL));
    });

    afterEach(() => {
      scope.stop();
    });

    it('Promise gets resolved on success.', (done) => {
      instance.parity.accountsInfo().then(s => {
        instance.parity.unsubscribe(s).then(b => {
          expect(b).to.be.true;
        });
      });
      done();
    });
  });

  describe('chainStatus', () => {
    beforeEach(() => {
      scope = mockWs([{ method: 'parity_subscribe', reply: 2, subscription: {
        method: 'parity_subscription',
        params: {
          result: {
            'blockGap': [0x123, 0x456]
          },
          subscription: 2
        }
      } }]);
      instance = new Pubsub(new Ws(TEST_WS_URL));
    });

    afterEach(() => {
      scope.stop();
    });

    it('retrieves the chain status', (done) => {
      instance.parity.chainStatus((error, result) => {
        expect(error).to.be.null;
        expect(result).to.deep.equal({
          'blockGap': [new BigNumber(0x123), new BigNumber(0x456)]
        });
        done();
      });
    });
  });

  describe('gasFloorTarget', () => {
    beforeEach(() => {
      scope = mockWs([{ method: 'parity_subscribe', reply: 2, subscription: {
        method: 'parity_subscription',
        params: {
          result: '0x123456',
          subscription: 2
        }
      } }]);
      instance = new Pubsub(new Ws(TEST_WS_URL));
    });

    afterEach(() => {
      scope.stop();
    });

    it('returns the gasfloor, formatted', (done) => {
      instance.parity.gasFloorTarget((error, result) => {
        expect(error).to.be.null;
        expect(isBigNumber(result)).to.be.true;
        expect(result.eq(0x123456)).to.be.true;
        done();
      });
    });
  });

  describe('transactionsLimit', () => {
    beforeEach(() => {
      scope = mockWs([{ method: 'parity_subscribe', reply: 2, subscription: {
        method: 'parity_subscription',
        params: {
          result: 1024,
          subscription: 2
        }
      } }]);
      instance = new Pubsub(new Ws(TEST_WS_URL));
    });

    afterEach(() => {
      scope.stop();
    });

    it('returns the tx limit, formatted', (done) => {
      instance.parity.transactionsLimit((error, result) => {
        expect(error).to.be.null;
        expect(isBigNumber(result)).to.be.true;
        expect(result.eq(1024)).to.be.true;
        done();
      });
    });
  });

  describe('minGasPrice', () => {
    beforeEach(() => {
      scope = mockWs([{ method: 'parity_subscribe', reply: 2, subscription: {
        method: 'parity_subscription',
        params: {
          result: '0x123456',
          subscription: 2
        }
      } }]);
      instance = new Pubsub(new Ws(TEST_WS_URL));
    });

    afterEach(() => {
      scope.stop();
    });

    it('returns the min gasprice, formatted', (done) => {
      instance.parity.minGasPrice((error, result) => {
        expect(error).to.be.null;
        expect(isBigNumber(result)).to.be.true;
        expect(result.eq(0x123456)).to.be.true;
        done();
      });
    });
  });

  describe('netPeers', () => {
    beforeEach(() => {
      scope = mockWs([{ method: 'parity_subscribe', reply: 2, subscription: {
        method: 'parity_subscription',
        params: {
          result: { active: 123, connected: 456, max: 789, peers: [] },
          subscription: 2
        }
      } }]);
      instance = new Pubsub(new Ws(TEST_WS_URL));
    });

    afterEach(() => {
      scope.stop();
    });

    it('returns the peer structure, formatted', (done) => {
      instance.parity.netPeers((error, peers) => {
        expect(error).to.be.null;
        expect(peers.active.eq(123)).to.be.true;
        expect(peers.connected.eq(456)).to.be.true;
        expect(peers.max.eq(789)).to.be.true;
        done();
      });
    });
  });

  describe('netPort', () => {
    beforeEach(() => {
      scope = mockWs([{ method: 'parity_subscribe', reply: 2, subscription: {
        method: 'parity_subscription',
        params: {
          result: 33030,
          subscription: 2
        }
      } }]);
      instance = new Pubsub(new Ws(TEST_WS_URL));
    });

    afterEach(() => {
      scope.stop();
    });

    it('returns the connected port, formatted', (done) => {
      instance.parity.netPort((error, count) => {
        expect(error).to.be.null;
        expect(isBigNumber(count)).to.be.true;
        expect(count.eq(33030)).to.be.true;
        done();
      });
    });
  });

// Eth API
  describe('accounts', () => {
    beforeEach(() => {
      scope = mockWs([{ method: 'parity_subscribe', reply: 2, subscription: {
        method: 'parity_subscription',
        params: {
          result: [address.toLowerCase()],
          subscription: 2
        }
      } }]);
      instance = new Pubsub(new Ws(TEST_WS_URL));
    });

    afterEach(() => {
      scope.stop();
    });

    it('returns a list of accounts, formatted', (done) => {
      instance.eth.accounts((error, accounts) => {
        expect(error).to.be.null;
        expect(accounts).to.deep.equal([address]);
        done();
      });
    });
  });

  describe('newHeads', () => {
    beforeEach(() => {
      scope = mockWs([{ method: 'eth_subscribe', reply: 2, subscription: {
        method: 'eth_subscription',
        params: {
          result: '0x123456',
          subscription: 2
        }
      } }]);
      instance = new Pubsub(new Ws(TEST_WS_URL));
    });

    afterEach(() => {
      scope.stop();
    });

    it('returns newHeads for eth_subscribe', (done) => {
      instance.eth.newHeads((error, blockNumber) => {
        expect(error).to.be.null;
        expect(blockNumber).to.equal('0x123456');
        done();
      });
    });
  });

  describe('blockNumber', () => {
    beforeEach(() => {
      scope = mockWs([{ method: 'parity_subscribe', reply: 2, subscription: {
        method: 'parity_subscription',
        params: {
          result: '0x123456',
          subscription: 2
        }
      } }]);
      instance = new Pubsub(new Ws(TEST_WS_URL));
    });

    afterEach(() => {
      scope.stop();
    });

    it('returns the current blockNumber, formatted', (done) => {
      instance.eth.blockNumber((error, blockNumber) => {
        expect(error).to.be.null;
        expect(isBigNumber(blockNumber)).to.be.true;
        expect(blockNumber.toString(16)).to.equal('123456');
        done();
      });
    });
  });

  describe('call', () => {
    beforeEach(() => {
      scope = mockWs([{ method: 'parity_subscribe', reply: 2, subscription: {
        method: 'parity_subscription',
        params: {
          result: [],
          subscription: 2
        }
      } }]);
      instance = new Pubsub(new Ws(TEST_WS_URL));
    });

    afterEach(() => {
      scope.stop();
    });

    it('formats the input options & blockNumber', (done) => {
      instance.eth.call((error) => {
        expect(error).to.be.null;
        expect(scope.body.parity_subscribe.params).to.deep.equal(['eth_call', [{ data: '0x12345678' }, 'earliest']]);
        done();
      }, { data: '12345678' }, 'earliest');
    });

    it('provides a latest blockNumber when not specified', (done) => {
      instance.eth.call((error) => {
        expect(error).to.be.null;
        expect(scope.body.parity_subscribe.params).to.deep.equal(['eth_call', [{ data: '0x12345678' }, 'latest']]);
        done();
      }, { data: '12345678' });
    });
  });

  describe('coinbase', () => {
    beforeEach(() => {
      scope = mockWs([{ method: 'parity_subscribe', reply: 2, subscription: {
        method: 'parity_subscription',
        params: {
          result: address.toLowerCase(),
          subscription: 2
        }
      } }]);
      instance = new Pubsub(new Ws(TEST_WS_URL));
    });

    afterEach(() => {
      scope.stop();
    });

    it('returns the coinbase, formatted', (done) => {
      instance.eth.coinbase((error, account) => {
        expect(error).to.be.null;
        expect(account).to.deep.equal(address);
        done();
      });
    });
  });

  describe('estimateGas', () => {
    beforeEach(() => {
      scope = mockWs([{ method: 'parity_subscribe', reply: 2, subscription: {
        method: 'parity_subscription',
        params: {
          result: '0x123',
          subscription: 2
        }
      } }]);
      instance = new Pubsub(new Ws(TEST_WS_URL));
    });

    afterEach(() => {
      scope.stop();
    });

    it('converts the options correctly', (done) => {
      instance.eth.estimateGas((error) => {
        expect(error).to.be.null;
        expect(scope.body.parity_subscribe.params).to.deep.equal(['eth_estimateGas', [{ gas: '0x5208' }]]);
        done();
      }, { gas: 21000 });
    });

    it('returns the gas used, formatted', (done) => {
      instance.eth.estimateGas((error, gas) => {
        expect(error).to.be.null;
        expect(isBigNumber(gas)).to.be.true;
        expect(gas.toString(16)).to.deep.equal('123');
        done();
      });
    });
  });

  describe('gasPrice', () => {
    beforeEach(() => {
      scope = mockWs([{ method: 'parity_subscribe', reply: 2, subscription: {
        method: 'parity_subscription',
        params: {
          result: '0x123',
          subscription: 2
        }
      } }]);
      instance = new Pubsub(new Ws(TEST_WS_URL));
    });

    afterEach(() => {
      scope.stop();
    });

    it('returns the gas price, formatted', (done) => {
      instance.eth.gasPrice((error, price) => {
        expect(error).to.be.null;
        expect(isBigNumber(price)).to.be.true;
        expect(price.toString(16)).to.deep.equal('123');
        done();
      });
    });
  });

  describe('getBalance', () => {
    beforeEach(() => {
      scope = mockWs([{ method: 'parity_subscribe', reply: 2, subscription: {
        method: 'parity_subscription',
        params: {
          result: '0x123',
          subscription: 2
        }
      } }]);
      instance = new Pubsub(new Ws(TEST_WS_URL));
    });

    afterEach(() => {
      scope.stop();
    });

    it('passes in the address (default blockNumber)', (done) => {
      instance.eth.getBalance((error) => {
        expect(error).to.be.null;
        expect(scope.body.parity_subscribe.params).to.deep.equal(['eth_getBalance', [address.toLowerCase(), 'latest']]);
        done();
      }, address);
    });

    it('passes in the address & blockNumber', (done) => {
      instance.eth.getBalance((error) => {
        expect(error).to.be.null;
        expect(scope.body.parity_subscribe.params).to.deep.equal(['eth_getBalance', [address.toLowerCase(), '0x456']]);
        done();
      }, address, 0x456);
    });

    it('returns the balance', (done) => {
      instance.eth.getBalance((error, balance) => {
        expect(error).to.be.null;
        expect(isBigNumber(balance)).to.be.true;
        expect(balance.toString(16)).to.deep.equal('123');
        done();
      }, address);
    });
  });

  describe('getBlockByHash', () => {
    beforeEach(() => {
      scope = mockWs([{ method: 'parity_subscribe', reply: 2, subscription: {
        method: 'parity_subscription',
        params: {
          result: { miner: address.toLowerCase() },
          subscription: 2
        }
      } }]);
      instance = new Pubsub(new Ws(TEST_WS_URL));
    });

    afterEach(() => {
      scope.stop();
    });

    it('formats the input hash as a hash, default full', (done) => {
      instance.eth.getBlockByHash((error) => {
        expect(error).to.be.null;
        expect(scope.body.parity_subscribe.params).to.deep.equal(['eth_getBlockByHash', ['0x1234', false]]);
        done();
      }, '1234');
    });

    it('formats the input hash as a hash, full true', (done) => {
      instance.eth.getBlockByHash((error) => {
        expect(error).to.be.null;
        expect(scope.body.parity_subscribe.params).to.deep.equal(['eth_getBlockByHash', ['0x1234', true]]);
        done();
      }, '1234', true);
    });

    it('formats the output into block', (done) => {
      instance.eth.getBlockByHash((error, block) => {
        expect(error).to.be.null;
        expect(block.miner).to.equal(address);
        done();
      }, '1234');
    });
  });

  describe('getBlockByNumber', () => {
    beforeEach(() => {
      scope = mockWs([{ method: 'parity_subscribe', reply: 2, subscription: {
        method: 'parity_subscription',
        params: {
          result: { miner: address.toLowerCase() },
          subscription: 2
        }
      } }]);
      instance = new Pubsub(new Ws(TEST_WS_URL));
    });

    afterEach(() => {
      scope.stop();
    });

    it('assumes blockNumber latest & full false', (done) => {
      instance.eth.getBlockByNumber((error) => {
        expect(error).to.be.null;
        expect(scope.body.parity_subscribe.params).to.deep.equal(['eth_getBlockByNumber', ['latest', false]]);
        done();
      });
    });

    it('uses input blockNumber & full false', (done) => {
      instance.eth.getBlockByNumber((error) => {
        expect(error).to.be.null;
        expect(scope.body.parity_subscribe.params).to.deep.equal(['eth_getBlockByNumber', ['0x1234', false]]);
        done();
      }, '0x1234');
    });

    it('formats the input blockNumber, full true', (done) => {
      instance.eth.getBlockByNumber((error) => {
        expect(error).to.be.null;
        expect(scope.body.parity_subscribe.params).to.deep.equal(['eth_getBlockByNumber', ['0x1234', true]]);
        done();
      }, 0x1234, true);
    });

    it('formats the output into block', (done) => {
      instance.eth.getBlockByNumber((error, block) => {
        expect(error).to.be.null;
        expect(block.miner).to.equal(address);
        done();
      }, 0x1234);
    });
  });

  describe('getTransactionCount', () => {
    beforeEach(() => {
      scope = mockWs([{ method: 'parity_subscribe', reply: 2, subscription: {
        method: 'parity_subscription',
        params: {
          result: '0x123',
          subscription: 2
        }
      } }]);
      instance = new Pubsub(new Ws(TEST_WS_URL));
    });

    afterEach(() => {
      scope.stop();
    });

    it('passes in the address (default blockNumber)', (done) => {
      instance.eth.getTransactionCount((error) => {
        expect(error).to.be.null;
        expect(scope.body.parity_subscribe.params).to.deep.equal(['eth_getTransactionCount', [address.toLowerCase(), 'latest']]);
        done();
      }, address);
    });

    it('passes in the address & blockNumber', (done) => {
      instance.eth.getTransactionCount((error) => {
        expect(error).to.be.null;
        expect(scope.body.parity_subscribe.params).to.deep.equal(['eth_getTransactionCount', [address.toLowerCase(), '0x456']]);
        done();
      }, address, 0x456);
    });

    it('returns the count, formatted', (done) => {
      instance.eth.getTransactionCount((error, count) => {
        expect(error).to.be.null;
        expect(isBigNumber(count)).to.be.true;
        expect(count.toString(16)).to.equal('123');
        done();
      }, address, 0x456);
    });
  });
});
