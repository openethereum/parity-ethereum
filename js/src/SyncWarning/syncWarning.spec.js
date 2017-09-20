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

import { shallow } from 'enzyme';
import React from 'react';

import SyncWarning from './';

let component;

function createRedux (syncing = null) {
  return {
    dispatch: () => {},
    subscribe: () => {},
    getState: () => {
      return {
        nodeStatus: {
          health: {
            overall: {
              status: syncing ? 'needsAttention' : 'ok',
              message: []
            }
          }
        }
      };
    }
  };
}

function render (store) {
  component = shallow(
    <SyncWarning />,
    { context: { store: store || createRedux() } }
  ).find('SyncWarning').shallow();

  return component;
}

describe('SyncWarning', () => {
  it('renders defaults', () => {
    expect(render()).to.be.ok;
  });

  it('does render when syncing', () => {
    expect(render(createRedux({})).find('div')).to.have.length.gte(1);
  });

  it('does not render when not syncing', () => {
    expect(render(createRedux(false)).find('div')).to.have.length(0);
  });
});
