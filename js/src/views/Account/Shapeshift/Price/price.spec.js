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

import Price from './';

let component;

function render (props = {}) {
  component = shallow(
    <Price
      coinSymbol='BTC'
      price={ { rate: 0.1, minimum: 0.1, limit: 0.9 } }
      error={ new Error('testing') }
      { ...props }
    />
  );

  return component;
}

describe('views/Account/Shapeshift/Price', () => {
  it('renders defaults', () => {
    expect(render()).to.be.ok;
  });
});
