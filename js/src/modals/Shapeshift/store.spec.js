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

import sinon from 'sinon';

import Store, { STAGE_COMPLETED, STAGE_OPTIONS, STAGE_WAIT_DEPOSIT, STAGE_WAIT_EXCHANGE, WARNING_NONE, WARNING_NO_PRICE } from './store';

const ADDRESS = '0xabcdeffdecbaabcdeffdecbaabcdeffdecbaabcdeffdecba';

describe('modals/Shapeshift/Store', () => {
  let store;

  beforeEach(() => {
    store = new Store(ADDRESS);
  });

  it('stores the ETH address', () => {
    expect(store.address).to.equal(ADDRESS);
  });

  it('defaults to BTC-ETH pair', () => {
    expect(store.coinSymbol).to.equal('BTC');
    expect(store.coinPair).to.equal('btc_eth');
  });

  it('defaults to stage STAGE_OPTIONS', () => {
    expect(store.stage).to.equal(STAGE_OPTIONS);
  });

  it('defaults to terms not accepted', () => {
    expect(store.hasAcceptedTerms).to.be.false;
  });

  describe('@action', () => {
    describe('setCoins', () => {
      it('sets the available coins', () => {
        const coins = ['BTC', 'ETC', 'XMR'];

        store.setCoins(coins);
        expect(store.coins.peek()).to.deep.equal(coins);
      });
    });

    describe('setCoinSymbol', () => {
      beforeEach(() => {
        sinon.stub(store, 'getCoinPrice');
        store.setCoinSymbol('XMR');
      });

      afterEach(() => {
        store.getCoinPrice.restore();
      });

      it('sets the coinSymbol', () => {
        expect(store.coinSymbol).to.equal('XMR');
      });

      it('sets the coinPair', () => {
        expect(store.coinPair).to.equal('xmr_eth');
      });

      it('resets the price retrieved', () => {
        expect(store.price).to.be.null;
      });

      it('retrieves the pair price', () => {
        expect(store.getCoinPrice).to.have.been.called;
      });
    });

    describe('setDepositAddress', () => {
      it('sets the depositAddress', () => {
        store.setDepositAddress('testing');
        expect(store.depositAddress).to.equal('testing');
      });
    });

    describe('setDepositInfo', () => {
      beforeEach(() => {
        store.setDepositInfo('testing');
      });

      it('sets the depositInfo', () => {
        expect(store.depositInfo).to.equal('testing');
      });

      it('sets the stage to STAGE_WAIT_EXCHANGE', () => {
        expect(store.stage).to.equal(STAGE_WAIT_EXCHANGE);
      });
    });

    describe('setError', () => {
      it('sets the error', () => {
        store.setError(new Error('testing'));
        expect(store.error).to.match(/testing/);
      });
    });

    describe('setExchangeInfo', () => {
      beforeEach(() => {
        store.setExchangeInfo('testing');
      });

      it('sets the exchangeInfo', () => {
        expect(store.exchangeInfo).to.equal('testing');
      });

      it('sets the stage to STAGE_COMPLETED', () => {
        expect(store.stage).to.equal(STAGE_COMPLETED);
      });
    });

    describe('setPrice', () => {
      it('sets the price', () => {
        store.setPrice('testing');
        expect(store.price).to.equal('testing');
      });

      it('clears any warnings once set', () => {
        store.setWarning(-999);
        store.setPrice('testing');
        expect(store.warning).to.equal(WARNING_NONE);
      });
    });

    describe('setRefundAddress', () => {
      it('sets the price', () => {
        store.setRefundAddress('testing');
        expect(store.refundAddress).to.equal('testing');
      });
    });

    describe('setStage', () => {
      it('sets the state', () => {
        store.setStage('testing');
        expect(store.stage).to.equal('testing');
      });
    });

    describe('setWarning', () => {
      it('sets the warning', () => {
        store.setWarning(-999);

        expect(store.warning).to.equal(-999);
      });

      it('clears the warning with no parameters', () => {
        store.setWarning(-999);
        store.setWarning();

        expect(store.warning).to.equal(WARNING_NONE);
      });
    });

    describe('toggleAcceptTerms', () => {
      it('changes state on hasAcceptedTerms', () => {
        store.toggleAcceptTerms();
        expect(store.hasAcceptedTerms).to.be.true;
      });
    });
  });

  describe('operations', () => {
    describe('getCoinPrice', () => {
      beforeEach(() => {
        sinon.stub(store._shapeshiftApi, 'getMarketInfo').resolves('testPrice');
        return store.getCoinPrice();
      });

      afterEach(() => {
        store._shapeshiftApi.getMarketInfo.restore();
      });

      it('retrieves the market info from ShapeShift', () => {
        expect(store._shapeshiftApi.getMarketInfo).to.have.been.calledWith('btc_eth');
      });

      it('stores the price retrieved', () => {
        expect(store.price).to.equal('testPrice');
      });

      it('sets a warning on failure', () => {
        store._shapeshiftApi.getMarketInfo.restore();
        sinon.stub(store._shapeshiftApi, 'getMarketInfo').rejects('someError');

        return store.getCoinPrice().then(() => {
          expect(store.warning).to.equal(WARNING_NO_PRICE);
        });
      });
    });

    describe('retrieveCoins', () => {
      beforeEach(() => {
        sinon.stub(store._shapeshiftApi, 'getCoins').resolves({
          BTC: { symbol: 'BTC', status: 'available' },
          ETC: { symbol: 'ETC' },
          XMR: { symbol: 'XMR', status: 'available' }
        });
        sinon.stub(store, 'getCoinPrice');
        return store.retrieveCoins();
      });

      afterEach(() => {
        store._shapeshiftApi.getCoins.restore();
        store.getCoinPrice.restore();
      });

      it('retrieves the coins from ShapeShift', () => {
        expect(store._shapeshiftApi.getCoins).to.have.been.called;
      });

      it('sets the available coins', () => {
        expect(store.coins.peek()).to.deep.equal([
          { status: 'available', symbol: 'BTC' },
          { status: 'available', symbol: 'XMR' }
        ]);
      });

      it('retrieves the price once resolved', () => {
        expect(store.getCoinPrice).to.have.been.called;
      });
    });

    describe('shift', () => {
      beforeEach(() => {
        sinon.stub(store, 'subscribe').resolves();
        sinon.stub(store._shapeshiftApi, 'shift').resolves({ deposit: 'depositAddress' });
        store.setRefundAddress('refundAddress');

        return store.shift();
      });

      afterEach(() => {
        store.subscribe.restore();
        store._shapeshiftApi.shift.restore();
      });

      it('moves to stage STAGE_WAIT_DEPOSIT', () => {
        expect(store.stage).to.equal(STAGE_WAIT_DEPOSIT);
      });

      it('calls ShapeShift with the correct parameters', () => {
        expect(store._shapeshiftApi.shift).to.have.been.calledWith(ADDRESS, 'refundAddress', store.coinPair);
      });

      it('sets the depositAddress', () => {
        expect(store.depositAddress).to.equal('depositAddress');
      });

      it('subscribes to updates', () => {
        expect(store.subscribe).to.have.been.called;
      });

      it('sets error when shift fails', () => {
        store._shapeshiftApi.shift.restore();
        sinon.stub(store._shapeshiftApi, 'shift').rejects({ message: 'testingError' });

        return store.shift().then(() => {
          expect(store.error).to.match(/testingError/);
        });
      });
    });

    describe('subscribe', () => {
      beforeEach(() => {
        sinon.stub(store._shapeshiftApi, 'subscribe');
        store.setDepositAddress('depositAddress');
        return store.subscribe();
      });

      afterEach(() => {
        store._shapeshiftApi.subscribe.restore();
      });

      it('calls into the ShapeShift subscribe', () => {
        expect(store._shapeshiftApi.subscribe).to.have.been.calledWith('depositAddress', store.onExchangeInfo);
      });

      describe('onExchangeInfo', () => {
        it('sets the error when fatal error retrieved', () => {
          store.onExchangeInfo({ fatal: true, message: 'testing' });
          expect(store.error.message).to.equal('testing');
        });

        it('does not set the error when non-fatal error retrieved', () => {
          store.onExchangeInfo({ message: 'testing' });
          expect(store.error).to.be.null;
        });

        describe('status received', () => {
          const INFO = { status: 'received' };

          beforeEach(() => {
            store.onExchangeInfo(null, INFO);
          });

          it('sets the depositInfo', () => {
            expect(store.depositInfo).to.deep.equal(INFO);
          });

          it('only advanced depositInfo once', () => {
            store.onExchangeInfo(null, Object.assign({}, INFO, { state: 'secondTime' }));
            expect(store.depositInfo).to.deep.equal(INFO);
          });
        });

        describe('status completed', () => {
          const INFO = { status: 'complete' };

          beforeEach(() => {
            store.onExchangeInfo(null, INFO);
          });

          it('sets the depositInfo', () => {
            expect(store.exchangeInfo).to.deep.equal(INFO);
          });

          it('only advanced depositInfo once', () => {
            store.onExchangeInfo(null, Object.assign({}, INFO, { state: 'secondTime' }));
            expect(store.exchangeInfo).to.deep.equal(INFO);
          });
        });
      });
    });

    describe('unsubscribe', () => {
      beforeEach(() => {
        sinon.stub(store._shapeshiftApi, 'unsubscribe');
        store.setDepositAddress('depositAddress');
        return store.unsubscribe();
      });

      afterEach(() => {
        store._shapeshiftApi.unsubscribe.restore();
      });

      it('calls into the ShapeShift unsubscribe', () => {
        expect(store._shapeshiftApi.unsubscribe).to.have.been.calledWith('depositAddress');
      });
    });
  });
});
