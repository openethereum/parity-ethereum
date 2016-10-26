// Copyright 2015, 2016 Ethcore (UK) Ltd.
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

import { TEST_HTTP_URL, mockHttp } from '../../../../test/mockRpc';
import { isBigNumber } from '../../../../test/types';

import Http from '../../transport/http';
import Ethcore from './ethcore';

const instance = new Ethcore(new Http(TEST_HTTP_URL));

describe('api/rpc/Ethcore', () => {
  describe('gasFloorTarget', () => {
    it('returns the gasfloor, formatted', () => {
      mockHttp([{ method: 'ethcore_gasFloorTarget', reply: { result: '0x123456' } }]);

      return instance.gasFloorTarget().then((count) => {
        expect(isBigNumber(count)).to.be.true;
        expect(count.eq(0x123456)).to.be.true;
      });
    });
  });

  describe('minGasPrice', () => {
    it('returns the min gasprice, formatted', () => {
      mockHttp([{ method: 'ethcore_minGasPrice', reply: { result: '0x123456' } }]);

      return instance.minGasPrice().then((count) => {
        expect(isBigNumber(count)).to.be.true;
        expect(count.eq(0x123456)).to.be.true;
      });
    });
  });

  describe('netMaxPeers', () => {
    it('returns the max peers, formatted', () => {
      mockHttp([{ method: 'ethcore_netMaxPeers', reply: { result: 25 } }]);

      return instance.netMaxPeers().then((count) => {
        expect(isBigNumber(count)).to.be.true;
        expect(count.eq(25)).to.be.true;
      });
    });
  });

  describe('newPeers', () => {
    it('returns the peer structure, formatted', () => {
      mockHttp([{ method: 'ethcore_netPeers', reply: { result: { active: 123, connected: 456, max: 789 } } }]);

      return instance.netPeers().then((peers) => {
        expect(peers.active.eq(123)).to.be.true;
        expect(peers.connected.eq(456)).to.be.true;
        expect(peers.max.eq(789)).to.be.true;
      });
    });
  });

  describe('netPort', () => {
    it('returns the connected port, formatted', () => {
      mockHttp([{ method: 'ethcore_netPort', reply: { result: 33030 } }]);

      return instance.netPort().then((count) => {
        expect(isBigNumber(count)).to.be.true;
        expect(count.eq(33030)).to.be.true;
      });
    });
  });

  describe('transactionsLimit', () => {
    it('returns the tx limit, formatted', () => {
      mockHttp([{ method: 'ethcore_transactionsLimit', reply: { result: 1024 } }]);

      return instance.transactionsLimit().then((count) => {
        expect(isBigNumber(count)).to.be.true;
        expect(count.eq(1024)).to.be.true;
      });
    });
  });
});
