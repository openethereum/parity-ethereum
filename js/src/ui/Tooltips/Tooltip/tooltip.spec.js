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

import React from 'react';
import { shallow } from 'enzyme';
import sinon from 'sinon';

import Tooltip from './';

let component;
let store;

function createRedux (currentId = 0) {
  store = {
    dispatch: sinon.stub(),
    subscribe: sinon.stub(),
    getState: () => {
      return {
        tooltip: {
          currentId,
          maxId: 2
        }
      };
    }
  };

  return store;
}

function render () {
  component = shallow(
    <Tooltip />,
    {
      context: {
        store: createRedux()
      }
    }
  ).find('Tooltip').shallow();

  return component;
}

describe('ui/Tooltips/Tooltip', () => {
  beforeEach(() => {
    render();
  });

  it('renders defaults', () => {
    expect(component.get(0)).to.be.ok;
  });

  it('renders null when id !== currentId', () => {
    expect(render(1).get(0)).to.be.null;
  });
});
