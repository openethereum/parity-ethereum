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

import { mockget, mockpost, shapeshift } from './helpers.spec.js';

describe('shapeshift/calls', () => {
  describe('getCoins', () => {
    const REPLY = {
      BTC: {
        name: 'Bitcoin',
        symbol: 'BTC',
        image: 'https://shapeshift.io/images/coins/bitcoin.png',
        status: 'available'
      },
      ETH: {
        name: 'Ether',
        symbol: 'ETH',
        image: 'https://shapeshift.io/images/coins/ether.png',
        status: 'available'
      }
    };

    let scope;

    before(() => {
      scope = mockget([{ path: 'getcoins', reply: REPLY }]);

      return shapeshift.getCoins();
    });

    it('makes the call', () => {
      expect(scope.isDone()).to.be.ok;
    });
  });

  describe('getMarketInfo', () => {
    const REPLY = {
      pair: 'btc_ltc',
      rate: 128.17959917,
      minerFee: 0.003,
      limit: 0,
      minimum: 0.00004632
    };

    let scope;

    before(() => {
      scope = mockget([{ path: 'marketinfo/btc_ltc', reply: REPLY }]);

      return shapeshift.getMarketInfo('btc_ltc');
    });

    it('makes the call', () => {
      expect(scope.isDone()).to.be.ok;
    });
  });

  describe('getStatus', () => {
    const REPLY = {
      status: '0x123',
      address: '0x123'
    };

    let scope;

    before(() => {
      scope = mockget([{ path: 'txStat/0x123', reply: REPLY }]);

      return shapeshift.getStatus('0x123');
    });

    it('makes the call', () => {
      expect(scope.isDone()).to.be.ok;
    });
  });

  describe('shift', () => {
    const REPLY = {
      deposit: '1BTC',
      depositType: 'btc',
      withdrawal: '0x456',
      withdrawalType: 'eth'
    };

    let scope;

    before(() => {
      scope = mockpost([{ path: 'shift', reply: REPLY }]);

      return shapeshift.shift('0x456', '1BTC', 'btc_eth');
    });

    it('makes the call', () => {
      expect(scope.isDone()).to.be.ok;
    });

    describe('body', () => {
      it('has withdrawal set', () => {
        expect(scope.body.shift.withdrawal).to.equal('0x456');
      });

      it('has returnAddress set', () => {
        expect(scope.body.shift.returnAddress).to.equal('1BTC');
      });

      it('has pair set', () => {
        expect(scope.body.shift.pair).to.equal('btc_eth');
      });
    });
  });
});
