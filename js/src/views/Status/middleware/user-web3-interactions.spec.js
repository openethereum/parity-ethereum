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

import sinon from 'sinon';
import WebInteractions from './user-web3-interactions';
import * as MiningActions from '../actions/modify-mining';

describe('MIDDLEWARE: WEB3 INTERACTIONS', () => {
  let cut;

  beforeEach('Mock cut', () => {
    const web3 = null;
    const ethcoreWeb3 = {
      setExtraData: sinon.spy()
    };
    cut = new WebInteractions(web3, ethcoreWeb3);
  });

  it('should get correct function names', () => {
    expect(cut.getMethod('modify minGasPrice')).to.equal('setMinGasPrice');
  });

  it('should not invoke web3 when a non modify action is dispatched', () => {
    // given
    const store = null;
    const next = sinon.spy();
    const middleware = cut.toMiddleware()(store)(next);
    const action = { type: 'testAction', payload: 'testPayload' };
    expect(middleware).to.be.a('function');
    expect(action).to.be.an('object');

    // when
    middleware(action);

    // then
    expect(next.calledWith(action)).to.be.true;
    Object.keys(cut.ethcoreWeb3).map(func => {
      expect(cut.ethcoreWeb3[func].notCalled).to.be.true;
    });
  });

  it('should invoke web3 when a modify action is dispatched', () => {
    // given
    const extraData = 'Parity';
    const store = null;
    const next = sinon.spy();
    const middleware = cut.toMiddleware()(store)(next);
    const action = MiningActions.modifyExtraData(extraData);
    expect(middleware).to.be.a('function');
    expect(action).to.be.an('object');

    // when
    middleware(action);

    // then
    expect(
      cut.ethcoreWeb3[cut.getMethod('modify extraData')]
      .calledWith(action.payload)
    ).to.be.true;
    expect(action.type).to.equal('update extraData');
    expect(next.calledWith({
      type: 'update extraData',
      payload: extraData
    })).to.be.true;
  });
});
