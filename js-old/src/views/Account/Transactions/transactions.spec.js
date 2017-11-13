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

import { ADDRESS, createApi, createRedux } from './transactions.test.js';

import Transactions from './';

let component;
let instance;

function render (props) {
  component = shallow(
    <Transactions
      address={ ADDRESS }
      { ...props }
    />,
    { context: { store: createRedux() } }
  ).find('Transactions').shallow({ context: { api: createApi() } });
  instance = component.instance();

  return component;
}

describe('views/Account/Transactions', () => {
  it('renders defaults', () => {
    expect(render()).to.be.ok;
  });

  describe('renderTransactionList', () => {
    it('renders Loading when isLoading === true', () => {
      instance.store.setLoading(true);
      expect(instance.renderTransactionList().type).to.match(/Loading/);
    });

    it('renders TxList when isLoading === true', () => {
      instance.store.setLoading(false);
      expect(instance.renderTransactionList().type).to.match(/Connect/);
    });
  });
});
