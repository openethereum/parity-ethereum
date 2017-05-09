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

import SignerMiddleware from './signerMiddleware';

const ADDRESS = '0x3456789012345678901234567890123456789012';
const RAW_SIGNED = 'testSignResponse';
const NONCE = new BigNumber(0x123454321);
const TRANSACTION = {
  from: ADDRESS,
  nonce: NONCE
};
const PAYLOAD = {
  condition: 'testCondition',
  gas: 'testGas',
  gasPrice: 'testGasPrice',
  id: 'testId',
  password: 'testPassword',
  payload: {
    sendTransaction: TRANSACTION
  }
};
const ACTION = {
  payload: PAYLOAD
};

let api;
let clock;
let hwstore;
let middleware;
let store;

function createApi () {
  api = {
    net: {
      version: sinon.stub().resolves('2')
    },
    parity: {
      nextNonce: sinon.stub().resolves(NONCE)
    },
    signer: {
      confirmRequest: sinon.stub().resolves(true),
      confirmRequestRaw: sinon.stub().resolves(true),
      rejectRequest: sinon.stub().resolves(true)
    }
  };

  return api;
}

function createHwStore () {
  hwstore = {
    signLedger: sinon.stub().resolves(RAW_SIGNED),
    wallets: {
      [ADDRESS]: {
        address: ADDRESS,
        via: 'ledger'
      }
    }
  };

  return hwstore;
}

function createRedux () {
  return {
    dispatch: sinon.stub(),
    getState: () => {
      return {
        worker: {
          worker: null
        }
      };
    }
  };
}

function create () {
  clock = sinon.useFakeTimers();
  store = createRedux();
  middleware = new SignerMiddleware(createApi());

  return middleware;
}

function teardown () {
  clock.restore();
}

describe('redux/SignerMiddleware', () => {
  beforeEach(() => {
    create();
  });

  afterEach(() => {
    teardown();
  });

  describe('createNoncePromise', () => {
    it('resolves via transaction.nonce when available', () => {
      const nonce = new BigNumber('0xabc');

      return middleware.createNoncePromise({ nonce }).then((_nonce) => {
        expect(_nonce).to.equal(nonce);
      });
    });

    it('calls parity_nextNonce', () => {
      return middleware.createNoncePromise({ from: 'testing' }).then((nonce) => {
        expect(api.parity.nextNonce).to.have.been.calledWith('testing');
        expect(nonce).to.equal(NONCE);
      });
    });
  });

  describe('confirmLedgerTransaction', () => {
    beforeEach(() => {
      sinon.spy(middleware, 'createNoncePromise');
      middleware._hwstore = createHwStore();

      return middleware.confirmLedgerTransaction(store, PAYLOAD.id, TRANSACTION);
    });

    afterEach(() => {
      middleware.createNoncePromise.restore();
    });

    it('creates nonce via createNoncePromise', () => {
      expect(middleware.createNoncePromise).to.have.been.calledWith(TRANSACTION);
    });

    it('calls into hardware signLedger', () => {
      expect(hwstore.signLedger).to.have.been.calledWith(TRANSACTION);
    });

    it('confirms via signer_confirmRequestRaw', () => {
      expect(api.signer.confirmRequestRaw).to.have.been.calledWith(PAYLOAD.id, RAW_SIGNED);
    });
  });

  describe('onConfirmStart', () => {
    describe('normal accounts', () => {
      beforeEach(() => {
        return middleware.onConfirmStart(store, ACTION);
      });

      it('calls into signer_confirmRequest', () => {
        expect(api.signer.confirmRequest).to.have.been.calledWith(
          PAYLOAD.id,
          {
            condition: PAYLOAD.condition,
            gas: PAYLOAD.gas,
            gasPrice: PAYLOAD.gasPrice
          },
          PAYLOAD.password
        );
      });
    });

    describe('hardware accounts', () => {
      beforeEach(() => {
        sinon.spy(middleware, 'confirmLedgerTransaction');
        middleware._hwstore = createHwStore();

        return middleware.onConfirmStart(store, ACTION);
      });

      afterEach(() => {
        middleware.confirmLedgerTransaction.restore();
      });

      it('calls out to confirmLedgerTransaction', () => {
        expect(middleware.confirmLedgerTransaction).to.have.been.called;
      });
    });

    describe('json wallet accounts', () => {
      beforeEach(() => {
        sinon.spy(middleware, 'confirmWalletTransaction');

        return middleware.onConfirmStart(store, {
          payload: Object.assign({}, PAYLOAD, { wallet: 'testWallet' })
        });
      });

      afterEach(() => {
        middleware.confirmWalletTransaction.restore();
      });

      it('calls out to confirmWalletTransaction', () => {
        expect(middleware.confirmWalletTransaction).to.have.been.called;
      });
    });
  });

  describe('onRejectStart', () => {
    beforeEach(() => {
      return middleware.onRejectStart(store, { payload: 'testId' });
    });

    it('calls into signer_rejectRequest', () => {
      expect(api.signer.rejectRequest).to.have.been.calledWith('testId');
    });
  });
});
