const helpers = require('../helpers.spec.js');
const { mockget, mockpost, shapeshift } = helpers;

describe('lib/calls', () => {
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
