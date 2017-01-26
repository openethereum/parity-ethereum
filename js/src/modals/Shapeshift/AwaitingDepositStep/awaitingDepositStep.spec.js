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

import AwaitingDepositStep from './';

let component;

function render () {
  component = shallow(
    <AwaitingDepositStep
      store={ {
        coinSymbol: 'BTC',
        price: { rate: 0.001, minimum: 0, limit: 1.999 }
      } }
    />
  );

  return component;
}

describe('modals/Shapeshift/AwaitingDepositStep', () => {
  it('renders defaults', () => {
    expect(render()).to.be.ok;
  });

  it('displays waiting for address with empty depositAddress', () => {
    render();
    expect(component.find('FormattedMessage').props().id).to.match(/awaitingConfirmation/);
  });

  it('displays waiting for deposit with non-empty depositAddress', () => {
    render({ depositAddress: 'xyz' });
    expect(component.find('FormattedMessage').first().props().id).to.match(/awaitingDeposit/);
  });
});
