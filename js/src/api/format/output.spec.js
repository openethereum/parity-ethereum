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

import { outBlock, outAccountInfo, outAddress, outChainStatus, outDate, outHistogram, outHwAccountInfo, outNumber, outPeer, outPeers, outReceipt, outRecentDapps, outSyncing, outTransaction, outTrace, outVaultMeta } from './output';
import { isAddress, isBigNumber, isInstanceOf } from '../../../test/types';

describe('api/format/output', () => {
  const address = '0x63cf90d3f0410092fc0fca41846f596223979195';
  const checksum = '0x63Cf90D3f0410092FC0fca41846f596223979195';

  describe('outAccountInfo', () => {
    it('returns meta objects parsed', () => {
      expect(outAccountInfo(
        { '0x63cf90d3f0410092fc0fca41846f596223979195': {
          name: 'name', uuid: 'uuid', meta: '{"name":"456"}' }
        }
      )).to.deep.equal({
        '0x63Cf90D3f0410092FC0fca41846f596223979195': {
          name: 'name', uuid: 'uuid', meta: { name: '456' }
        }
      });
    });

    it('returns objects without meta & uuid as required', () => {
      expect(outAccountInfo(
        { '0x63cf90d3f0410092fc0fca41846f596223979195': { name: 'name' } }
      )).to.deep.equal({
        '0x63Cf90D3f0410092FC0fca41846f596223979195': { name: 'name' }
      });
    });
  });

  describe('outAddress', () => {
    it('retuns the address as checksummed', () => {
      expect(outAddress(address)).to.equal(checksum);
    });

    it('retuns the checksum as checksummed', () => {
      expect(outAddress(checksum)).to.equal(checksum);
    });
  });

  describe('outBlock', () => {
    ['author', 'miner'].forEach((input) => {
      it(`formats ${input} address as address`, () => {
        const block = {};

        block[input] = address;
        const formatted = outBlock(block)[input];

        expect(isAddress(formatted)).to.be.true;
        expect(formatted).to.equal(checksum);
      });
    });

    ['difficulty', 'gasLimit', 'gasUsed', 'number', 'nonce', 'totalDifficulty'].forEach((input) => {
      it(`formats ${input} number as hexnumber`, () => {
        const block = {};

        block[input] = 0x123;
        const formatted = outBlock(block)[input];

        expect(isInstanceOf(formatted, BigNumber)).to.be.true;
        expect(formatted.toString(16)).to.equal('123');
      });
    });

    ['timestamp'].forEach((input) => {
      it(`formats ${input} number as Date`, () => {
        const block = {};

        block[input] = 0x57513668;
        const formatted = outBlock(block)[input];

        expect(isInstanceOf(formatted, Date)).to.be.true;
        expect(formatted.getTime()).to.equal(1464940136000);
      });
    });

    it('ignores and passes through unknown keys', () => {
      expect(outBlock({ someRandom: 'someRandom' })).to.deep.equal({ someRandom: 'someRandom' });
    });

    it('formats a block with all the info converted', () => {
      expect(
        outBlock({
          author: address,
          miner: address,
          difficulty: '0x100',
          gasLimit: '0x101',
          gasUsed: '0x102',
          number: '0x103',
          nonce: '0x104',
          totalDifficulty: '0x105',
          timestamp: '0x57513668',
          extraData: 'someExtraStuffInHere'
        })
      ).to.deep.equal({
        author: checksum,
        miner: checksum,
        difficulty: new BigNumber('0x100'),
        gasLimit: new BigNumber('0x101'),
        gasUsed: new BigNumber('0x102'),
        number: new BigNumber('0x103'),
        nonce: new BigNumber('0x104'),
        totalDifficulty: new BigNumber('0x105'),
        timestamp: new Date('2016-06-03T07:48:56.000Z'),
        extraData: 'someExtraStuffInHere'
      });
    });
  });

  describe('outChainStatus', () => {
    it('formats blockGap values', () => {
      const status = {
        blockGap: [0x1234, '0x5678']
      };

      expect(outChainStatus(status)).to.deep.equal({
        blockGap: [new BigNumber(0x1234), new BigNumber(0x5678)]
      });
    });

    it('handles null blockGap values', () => {
      const status = {
        blockGap: null
      };

      expect(outChainStatus(status)).to.deep.equal(status);
    });
  });

  describe('outDate', () => {
    it('converts a second date in unix timestamp', () => {
      expect(outDate(0x57513668)).to.deep.equal(new Date('2016-06-03T07:48:56.000Z'));
    });
  });

  describe('outHistogram', () => {
    ['bucketBounds', 'counts'].forEach((type) => {
      it(`formats ${type} as number arrays`, () => {
        expect(
          outHistogram({ [type]: [0x123, 0x456, 0x789] })
        ).to.deep.equal({
          [type]: [new BigNumber(0x123), new BigNumber(0x456), new BigNumber(0x789)]
        });
      });
    });
  });

  describe('outHwAccountInfo', () => {
    it('returns objects with formatted addresses', () => {
      expect(outHwAccountInfo(
        { '0x63cf90d3f0410092fc0fca41846f596223979195': { manufacturer: 'mfg', name: 'type' } }
      )).to.deep.equal({
        '0x63Cf90D3f0410092FC0fca41846f596223979195': { manufacturer: 'mfg', name: 'type' }
      });
    });
  });

  describe('outNumber', () => {
    it('returns a BigNumber equalling the value', () => {
      const bn = outNumber('0x123456');

      expect(isBigNumber(bn)).to.be.true;
      expect(bn.eq(0x123456)).to.be.true;
    });

    it('assumes 0 when ivalid input', () => {
      expect(outNumber().eq(0)).to.be.true;
    });
  });

  describe('outPeer', () => {
    it('converts all internal numbers to BigNumbers', () => {
      expect(outPeer({
        caps: ['par/1'],
        id: '0x01',
        name: 'Parity',
        network: {
          localAddress: '10.0.0.1',
          remoteAddress: '10.0.0.1'
        },
        protocols: {
          par: {
            difficulty: '0x0f',
            head: '0x02',
            version: 63
          }
        }
      })).to.deep.equal({
        caps: ['par/1'],
        id: '0x01',
        name: 'Parity',
        network: {
          localAddress: '10.0.0.1',
          remoteAddress: '10.0.0.1'
        },
        protocols: {
          par: {
            difficulty: new BigNumber(15),
            head: '0x02',
            version: 63
          }
        }
      });
    });

    it('does not output null protocols', () => {
      expect(outPeer({
        caps: ['par/1'],
        id: '0x01',
        name: 'Parity',
        network: {
          localAddress: '10.0.0.1',
          remoteAddress: '10.0.0.1'
        },
        protocols: {
          les: null
        }
      })).to.deep.equal({
        caps: ['par/1'],
        id: '0x01',
        name: 'Parity',
        network: {
          localAddress: '10.0.0.1',
          remoteAddress: '10.0.0.1'
        },
        protocols: {}
      });
    });
  });

  describe('outPeers', () => {
    it('converts all internal numbers to BigNumbers', () => {
      expect(outPeers({
        active: 789,
        connected: '456',
        max: 0x7b,
        peers: [
          {
            caps: ['par/1'],
            id: '0x01',
            name: 'Parity',
            network: {
              localAddress: '10.0.0.1',
              remoteAddress: '10.0.0.1'
            },
            protocols: {
              par: {
                difficulty: '0x0f',
                head: '0x02',
                version: 63
              },
              les: null
            }
          }
        ]
      })).to.deep.equal({
        active: new BigNumber(789),
        connected: new BigNumber(456),
        max: new BigNumber(123),
        peers: [
          {
            caps: ['par/1'],
            id: '0x01',
            name: 'Parity',
            network: {
              localAddress: '10.0.0.1',
              remoteAddress: '10.0.0.1'
            },
            protocols: {
              par: {
                difficulty: new BigNumber(15),
                head: '0x02',
                version: 63
              }
            }
          }
        ]
      });
    });
  });

  describe('outReceipt', () => {
    ['contractAddress'].forEach((input) => {
      it(`formats ${input} address as address`, () => {
        const block = {};

        block[input] = address;
        const formatted = outReceipt(block)[input];

        expect(isAddress(formatted)).to.be.true;
        expect(formatted).to.equal(checksum);
      });
    });

    ['blockNumber', 'cumulativeGasUsed', 'cumulativeGasUsed', 'gasUsed', 'transactionIndex'].forEach((input) => {
      it(`formats ${input} number as hexnumber`, () => {
        const block = {};

        block[input] = 0x123;
        const formatted = outReceipt(block)[input];

        expect(isInstanceOf(formatted, BigNumber)).to.be.true;
        expect(formatted.toString(16)).to.equal('123');
      });
    });

    it('ignores and passes through unknown keys', () => {
      expect(outReceipt({ someRandom: 'someRandom' })).to.deep.equal({ someRandom: 'someRandom' });
    });

    it('formats a receipt with all the info converted', () => {
      expect(
        outReceipt({
          contractAddress: address,
          blockNumber: '0x100',
          cumulativeGasUsed: '0x101',
          gasUsed: '0x102',
          transactionIndex: '0x103',
          extraData: 'someExtraStuffInHere'
        })
      ).to.deep.equal({
        contractAddress: checksum,
        blockNumber: new BigNumber('0x100'),
        cumulativeGasUsed: new BigNumber('0x101'),
        gasUsed: new BigNumber('0x102'),
        transactionIndex: new BigNumber('0x103'),
        extraData: 'someExtraStuffInHere'
      });
    });
  });

  describe('outRecentDapps', () => {
    it('formats the URLs with timestamps', () => {
      expect(outRecentDapps({ testing: 0x57513668 })).to.deep.equal({
        testing: new Date('2016-06-03T07:48:56.000Z')
      });
    });
  });

  describe('outSyncing', () => {
    ['currentBlock', 'highestBlock', 'startingBlock', 'warpChunksAmount', 'warpChunksProcessed'].forEach((input) => {
      it(`formats ${input} numbers as a number`, () => {
        expect(outSyncing({ [input]: '0x123' })).to.deep.equal({
          [input]: new BigNumber('0x123')
        });
      });
    });

    it('formats blockGap properly', () => {
      expect(outSyncing({ blockGap: [0x123, 0x456] })).to.deep.equal({
        blockGap: [new BigNumber(0x123), new BigNumber(0x456)]
      });
    });
  });

  describe('outTransaction', () => {
    ['from', 'to'].forEach((input) => {
      it(`formats ${input} address as address`, () => {
        const block = {};

        block[input] = address;
        const formatted = outTransaction(block)[input];

        expect(isAddress(formatted)).to.be.true;
        expect(formatted).to.equal(checksum);
      });
    });

    ['blockNumber', 'gasPrice', 'gas', 'minBlock', 'nonce', 'transactionIndex', 'value'].forEach((input) => {
      it(`formats ${input} number as hexnumber`, () => {
        const block = {};

        block[input] = 0x123;
        const formatted = outTransaction(block)[input];

        expect(isInstanceOf(formatted, BigNumber)).to.be.true;
        expect(formatted.toString(16)).to.equal('123');
      });
    });

    it('passes minBlock as null when null', () => {
      expect(outTransaction({ minBlock: null })).to.deep.equal({ minBlock: null });
    });

    it('ignores and passes through unknown keys', () => {
      expect(outTransaction({ someRandom: 'someRandom' })).to.deep.equal({ someRandom: 'someRandom' });
    });

    it('formats a transaction with all the info converted', () => {
      expect(
        outTransaction({
          from: address,
          to: address,
          blockNumber: '0x100',
          gasPrice: '0x101',
          gas: '0x102',
          nonce: '0x103',
          transactionIndex: '0x104',
          value: '0x105',
          extraData: 'someExtraStuffInHere'
        })
      ).to.deep.equal({
        from: checksum,
        to: checksum,
        blockNumber: new BigNumber('0x100'),
        gasPrice: new BigNumber('0x101'),
        gas: new BigNumber('0x102'),
        nonce: new BigNumber('0x103'),
        transactionIndex: new BigNumber('0x104'),
        value: new BigNumber('0x105'),
        extraData: 'someExtraStuffInHere'
      });
    });
  });

  describe('outTrace', () => {
    it('ignores and passes through unknown keys', () => {
      expect(outTrace({ someRandom: 'someRandom' })).to.deep.equal({ someRandom: 'someRandom' });
    });

    it('formats a trace with all the info converted', () => {
      const formatted = outTrace({
        type: 'call',
        action: {
          from: address,
          to: address,
          value: '0x06',
          gas: '0x07',
          input: '0x1234',
          callType: 'call'
        },
        result: {
          gasUsed: '0x08',
          output: '0x5678'
        },
        traceAddress: [ '0x2' ],
        subtraces: 3,
        transactionPosition: '0xb',
        transactionHash: '0x000000000000000000000000000000000000000000000000000000000000000c',
        blockNumber: '0x0d',
        blockHash: '0x000000000000000000000000000000000000000000000000000000000000000e'
      });

      expect(isBigNumber(formatted.action.gas)).to.be.true;
      expect(formatted.action.gas.toNumber()).to.equal(7);
      expect(isBigNumber(formatted.action.value)).to.be.true;
      expect(formatted.action.value.toNumber()).to.equal(6);

      expect(formatted.action.from).to.equal(checksum);
      expect(formatted.action.to).to.equal(checksum);

      expect(isBigNumber(formatted.blockNumber)).to.be.true;
      expect(formatted.blockNumber.toNumber()).to.equal(13);
      expect(isBigNumber(formatted.transactionPosition)).to.be.true;
      expect(formatted.transactionPosition.toNumber()).to.equal(11);
    });
  });

  describe('outVaultMeta', () => {
    it('returns an exmpt object on null', () => {
      expect(outVaultMeta(null)).to.deep.equal({});
    });

    it('returns the original value if not string', () => {
      expect(outVaultMeta({ test: 123 })).to.deep.equal({ test: 123 });
    });

    it('returns an object from JSON string', () => {
      expect(outVaultMeta('{"test":123}')).to.deep.equal({ test: 123 });
    });

    it('returns an empty object on invalid JSON', () => {
      expect(outVaultMeta('{"test"}')).to.deep.equal({});
    });
  });
});
