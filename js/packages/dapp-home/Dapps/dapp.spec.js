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

import Dapp from './dapp';

import { createStore } from './dapps.test.js';

let component;
let instance;
let store;

function render () {
  store = createStore();
  component = shallow(
    <Dapp
      id='testId'
      store={ store }
      timestamp={ Date.now() }
    />
  );
  instance = component.instance();

  return component;
}

describe('views/Home/Dapp', () => {
  beforeEach(() => {
    render();
    return instance.componentWillMount();
  });

  it('renders defaults', () => {
    expect(component).to.be.ok;
  });

  it('loads the dapp on mount', () => {
    expect(store.loadApp).to.have.been.calledWith('testId');
  });
});
