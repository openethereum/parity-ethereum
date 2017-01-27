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

import Contracts from '~/contracts';
import { initialState as defaultNodeStatusState } from './statusReducer';
import ChainMiddleware from './chainMiddleware';
import { createWsApi } from '~/../test/e2e/ethapi';

let middleware;
let next;
let store;

const api = createWsApi();

Contracts.create(api);

function createMiddleware (collection = {}) {
  middleware = new ChainMiddleware().toMiddleware();
  next = sinon.stub();
  store = {
    dispatch: sinon.stub(),
    getState: () => {
      return {
        api: api,
        nodeStatus: Object.assign({}, defaultNodeStatusState, collection)
      };
    }
  };

  return middleware;
}

function callMiddleware (action) {
  return middleware(store)(next)(action);
}

describe('reduxs/providers/ChainMiddleware', () => {
  describe('next action', () => {
    beforeEach(() => {
      createMiddleware();
    });

    it('calls next with matching actiontypes', () => {
      callMiddleware({ type: 'statusCollection' });

      expect(next).to.have.been.calledWithMatch({ type: 'statusCollection' });
    });

    it('calls next with non-matching actiontypes', () => {
      callMiddleware({ type: 'nonMatchingType' });

      expect(next).to.have.been.calledWithMatch({ type: 'nonMatchingType' });
    });
  });

  describe('chain switching', () => {
    it('does not dispatch when moving from the initial/unknown chain', () => {
      createMiddleware();
      callMiddleware({ type: 'statusCollection', collection: { netChain: 'homestead' } });

      expect(store.dispatch).not.to.have.been.called;
    });

    it('does not dispatch when moving to the same chain', () => {
      createMiddleware({ netChain: 'homestead' });
      callMiddleware({ type: 'statusCollection', collection: { netChain: 'homestead' } });

      expect(store.dispatch).not.to.have.been.called;
    });

    it('does dispatch when moving between chains', () => {
      createMiddleware({ netChain: 'homestead' });
      callMiddleware({ type: 'statusCollection', collection: { netChain: 'ropsten' } });

      expect(store.dispatch).to.have.been.called;
    });
  });
});
