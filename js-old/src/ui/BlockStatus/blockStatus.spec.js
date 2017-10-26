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
import { shallow } from 'enzyme';
import React from 'react';
import sinon from 'sinon';

import BlockStatus from './';

let component;

function createRedux (syncing = false, blockNumber = new BigNumber(123)) {
  return {
    dispatch: sinon.stub(),
    subscribe: sinon.stub(),
    getState: () => {
      return {
        nodeStatus: {
          blockNumber,
          syncing
        }
      };
    }
  };
}

function render (reduxStore = createRedux(), props) {
  component = shallow(
    <BlockStatus { ...props } />,
    { context: { store: reduxStore } }
  ).find('BlockStatus').shallow();

  return component;
}

describe('ui/BlockStatus', () => {
  it('renders defaults', () => {
    expect(render()).to.be.ok;
  });

  it('renders null with no blockNumber', () => {
    expect(render(createRedux(false, null)).find('div')).to.have.length(0);
  });

  it('renders only the best block when syncing === false', () => {
    const messages = render().find('FormattedMessage');

    expect(messages).to.have.length(1);
    expect(messages).to.have.id('ui.blockStatus.bestBlock');
  });

  it('renders only the warp restore status when restoring', () => {
    const messages = render(createRedux({
      warpChunksAmount: new BigNumber(100),
      warpChunksProcessed: new BigNumber(5)
    })).find('FormattedMessage');

    expect(messages).to.have.length(1);
    expect(messages).to.have.id('ui.blockStatus.warpRestore');
  });

  it('renders the current/highest when syncing', () => {
    const messages = render(createRedux({
      currentBlock: new BigNumber(123),
      highestBlock: new BigNumber(456)
    })).find('FormattedMessage');

    expect(messages).to.have.length(1);
    expect(messages).to.have.id('ui.blockStatus.syncStatus');
  });

  it('renders warp blockGap when catching up', () => {
    const messages = render(createRedux({
      blockGap: [new BigNumber(123), new BigNumber(456)]
    })).find('FormattedMessage');

    expect(messages).to.have.length(1);
    expect(messages).to.have.id('ui.blockStatus.warpStatus');
  });
});
