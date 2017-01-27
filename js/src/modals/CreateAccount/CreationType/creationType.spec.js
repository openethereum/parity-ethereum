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

import { createStore } from '../createAccount.test.js';

import CreationType from './';

let component;
let store;

function render () {
  store = createStore();
  component = shallow(
    <CreationType
      store={ store }
    />
  );

  return component;
}

describe('modals/CreateAccount/CreationType', () => {
  beforeEach(() => {
    render();
  });

  it('renders with defaults', () => {
    expect(component).to.be.ok;
  });

  describe('selector', () => {
    const SELECT_TYPE = 'fromRaw';
    let selector;

    beforeEach(() => {
      store.setCreateType(SELECT_TYPE);
      selector = component.find('RadioButtonGroup');
    });

    it('renders the selector', () => {
      expect(selector.get(0)).to.be.ok;
    });

    it('passes the store type to defaultSelected', () => {
      expect(selector.props().defaultSelected).to.equal(SELECT_TYPE);
    });
  });

  describe('events', () => {
    describe('onChange', () => {
      beforeEach(() => {
        component.instance().onChange({ target: { value: 'testing' } });
      });

      it('changes the store createType', () => {
        expect(store.createType).to.equal('testing');
      });
    });
  });
});
