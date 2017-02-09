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

import apiutil from '~/api/util';

import Balance from './';

const BALANCE = {
  tokens: [
    { value: '122', token: { tag: 'ETH' } },
    { value: '345', token: { tag: 'GAV', format: 1 } },
    { value: '0', token: { tag: 'TST', format: 1 } }
  ]
};

let api;
let component;

function createApi () {
  api = {
    dappsUrl: 'http://testDapps:1234/',
    util: apiutil
  };

  return api;
}

function render (props = {}) {
  if (!props.balance) {
    props.balance = BALANCE;
  }

  const api = createApi();

  component = shallow(
    <Balance
      className='testClass'
      { ...props }
    />,
    {
      context: { api }
    }
  );

  return component;
}

describe('ui/Balance', () => {
  beforeEach(() => {
    render();
  });

  it('renders defaults', () => {
    expect(component).to.be.ok;
  });

  it('passes the specified className', () => {
    expect(component.hasClass('testClass')).to.be.true;
  });

  it('renders all the non-zero balances', () => {
    expect(component.find('Connect(TokenImage)')).to.have.length(2);
  });

  describe('render specifiers', () => {
    it('renders only the single token with showOnlyEth', () => {
      render({ showOnlyEth: true });
      expect(component.find('Connect(TokenImage)')).to.have.length(1);
    });

    it('renders all the tokens with showZeroValues', () => {
      render({ showZeroValues: true });
      expect(component.find('Connect(TokenImage)')).to.have.length(3);
    });

    it('shows ETH with zero value with showOnlyEth & showZeroValues', () => {
      render({
        showOnlyEth: true,
        showZeroValues: true,
        balance: {
          tokens: [
            { value: '0', token: { tag: 'ETH' } },
            { value: '345', token: { tag: 'GAV', format: 1 } }
          ]
        }
      });
      expect(component.find('Connect(TokenImage)')).to.have.length(1);
    });
  });
});
