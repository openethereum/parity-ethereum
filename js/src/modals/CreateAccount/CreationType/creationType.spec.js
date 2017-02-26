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
let instance;
let store;

function render () {
  store = createStore();
  component = shallow(
    <CreationType
      store={ store }
    />
  );
  instance = component.instance();

  return component;
}

describe('modals/CreateAccount/CreationType', () => {
  beforeEach(() => {
    render();
  });

  it('renders with defaults', () => {
    expect(component).to.be.ok;
  });

  describe('events', () => {
    describe('onChange', () => {
      beforeEach(() => {
        instance.onChange({ key: 'testing' });
      });

      it('changes the store createType', () => {
        expect(store.createType).to.equal('testing');
      });
    });
  });
});
