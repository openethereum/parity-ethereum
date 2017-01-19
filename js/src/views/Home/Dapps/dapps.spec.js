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

import { shallow } from 'enzyme';
import React from 'react';

import Dapps from './';

import { createStore } from './dapps.test.js';

let component;
let store;

function render (history = []) {
  store = createStore();
  component = shallow(
    <Dapps
      history={ history }
      store={ store }
    />
  );

  return component;
}

describe('views/Home/Dapps', () => {
  it('renders defaults', () => {
    expect(render()).to.be.ok;
  });

  describe('no history', () => {
    beforeEach(() => {
      render();
    });

    it('renders empty message', () => {
      expect(component.find('FormattedMessage').props().id).to.equal('home.dapps.none');
    });
  });

  describe('with history', () => {
    const HISTORY = [
      { timestamp: 1, entry: 'testABC' },
      { timestamp: 2, entry: 'testDEF' }
    ];

    beforeEach(() => {
      render(HISTORY);
    });

    it('renders dapps', () => {
      expect(component.find('Dapp').length).to.equal(HISTORY.length);
    });

    it('has Dapp with dapp id', () => {
      expect(component.find('Dapp').at(0).props().id).to.equal(HISTORY[0].entry);
    });

    it('has Dapp with dapp timestamp', () => {
      expect(component.find('Dapp').at(1).props().timestamp).to.equal(HISTORY[1].timestamp);
    });
  });
});
