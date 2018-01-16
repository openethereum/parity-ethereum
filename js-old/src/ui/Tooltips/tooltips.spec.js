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
import sinon from 'sinon';

import Tooltips from './';

let component;
let router;
let store;

function createRedux () {
  store = {
    dispatch: sinon.stub(),
    subscribe: sinon.stub(),
    getState: () => {
      return {
        tooltip: {
          currentId: 1
        }
      };
    }
  };

  return store;
}

function createRouter () {
  router = {
    push: sinon.stub()
  };

  return router;
}

function render () {
  component = shallow(
    <Tooltips />,
    {
      context: {
        store: createRedux()
      }
    }
  ).find('Tooltips').shallow({
    context: {
      router: createRouter()
    }
  });

  return component;
}

describe('ui/Tooltips', () => {
  beforeEach(() => {
    render();
  });

  it('renders defaults', () => {
    expect(component.get(0)).to.be.ok;
  });
});
