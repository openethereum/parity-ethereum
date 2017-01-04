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

import Store from '../store';

import OptionsStep from './';

const ADDRESS = '0x1234567890123456789012345678901234567890';

let component;
let store;

function render () {
  store = new Store(ADDRESS);
  component = shallow(
    <OptionsStep store={ store } />
  );

  return component;
}

describe('modals/Shapeshift/OptionsStep', () => {
  it('renders defaults', () => {
    expect(render()).to.be.ok;
  });
});
