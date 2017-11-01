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

import Eth from './eth';

const START_BLOCK = 5000;

function stubApi (blockNumber) {
  const _calls = {
    blockNumber: []
  };

  return {
    _calls,
    transport: {
      isConnected: true
    },
    eth: {
      blockNumber: () => {
        const stub = sinon.stub().resolves(new BigNumber(blockNumber || START_BLOCK))();

        _calls.blockNumber.push(stub);
        return stub;
      }
    }
  };
}

describe('api/subscriptions/eth', () => {
  let api;
  let eth;
  let cb;

  beforeEach(() => {
    api = stubApi();
    cb = sinon.stub();
    eth = new Eth(cb, api);
  });

  describe('constructor', () => {
    it('starts the instance in a stopped state', () => {
      expect(eth.isStarted).to.be.false;
    });
  });

  describe('start', () => {
    describe('blockNumber available', () => {
      beforeEach(() => {
        return eth.start();
      });

      it('sets the started status', () => {
        expect(eth.isStarted).to.be.true;
      });

      it('calls eth_blockNumber', () => {
        expect(api._calls.blockNumber.length).to.be.ok;
      });

      it('updates subscribers', () => {
        expect(cb).to.have.been.calledWith('eth_blockNumber', null, new BigNumber(START_BLOCK));
      });
    });

    describe('blockNumber not available', () => {
      beforeEach(() => {
        api = stubApi(-1);
        eth = new Eth(cb, api);
        return eth.start();
      });

      it('sets the started status', () => {
        expect(eth.isStarted).to.be.true;
      });

      it('calls eth_blockNumber', () => {
        expect(api._calls.blockNumber.length).to.be.ok;
      });

      it('does not update subscribers', () => {
        expect(cb).not.to.been.called;
      });
    });
  });
});
