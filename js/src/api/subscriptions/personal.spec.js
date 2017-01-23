// Copyright 2015, 2016 Parity Technologies (UK) Ltd.
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

import Personal from './personal';

const TEST_INFO = {
  '0xfa64203C044691aA57251aF95f4b48d85eC00Dd5': {
    name: 'test'
  }
};
const TEST_LIST = ['0xfa64203C044691aA57251aF95f4b48d85eC00Dd5'];

function stubApi (accounts, info) {
  const _calls = {
    allAccountsInfo: [],
    listAccounts: []
  };

  return {
    _calls,
    parity: {
      allAccountsInfo: () => {
        const stub = sinon.stub().resolves(info || TEST_INFO)();

        _calls.allAccountsInfo.push(stub);
        return stub;
      }
    },
    eth: {
      accounts: () => {
        const stub = sinon.stub().resolves(accounts || TEST_LIST)();

        _calls.listAccounts.push(stub);
        return stub;
      }
    }
  };
}

function stubLogging () {
  return {
    subscribe: sinon.stub()
  };
}

describe('api/subscriptions/personal', () => {
  let api;
  let cb;
  let logging;
  let personal;

  beforeEach(() => {
    api = stubApi();
    cb = sinon.stub();
    logging = stubLogging();
    personal = new Personal(cb, api, logging);
  });

  describe('constructor', () => {
    it('starts the instance in a stopped state', () => {
      expect(personal.isStarted).to.be.false;
    });
  });

  describe('start', () => {
    describe('info available', () => {
      beforeEach(() => {
        return personal.start();
      });

      it('sets the started status', () => {
        expect(personal.isStarted).to.be.true;
      });

      it('calls parity_allAccountsInfo', () => {
        expect(api._calls.allAccountsInfo.length).to.be.ok;
      });

      it('calls eth_accounts', () => {
        expect(api._calls.listAccounts.length).to.be.ok;
      });

      it('updates subscribers', () => {
        expect(cb.firstCall).to.have.been.calledWith('eth_accounts', null, TEST_LIST);
        expect(cb.secondCall).to.have.been.calledWith('parity_allAccountsInfo', null, TEST_INFO);
      });
    });

    describe('info not available', () => {
      beforeEach(() => {
        api = stubApi([], {});
        personal = new Personal(cb, api, logging);
        return personal.start();
      });

      it('sets the started status', () => {
        expect(personal.isStarted).to.be.true;
      });

      it('calls personal_accountsInfo', () => {
        expect(api._calls.allAccountsInfo.length).to.be.ok;
      });

      it('calls personal_listAccounts', () => {
        expect(api._calls.listAccounts.length).to.be.ok;
      });
    });
  });
});
