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

import Accounts from './';

let component;

function render (props = {}) {
  component = shallow(
    <Accounts
      { ...props }
    />
  );

  return component;
}

describe('ui/VaultCard/Accounts', () => {
  beforeEach(() => {
    render();
  });

  it('renders empty when no accounts supplied', () => {
    expect(
      component.find('FormattedMessage').props().id
    ).to.equal('vaults.accounts.empty');
  });

  describe('with accounts', () => {
    const ACCOUNTS = ['0x123', '0x456'];
    let identities;

    beforeEach(() => {
      render({ accounts: ACCOUNTS });
      identities = component.find('IdentityIcon');
    });

    it('renders the accounts when supplied', () => {
      expect(identities).to.have.length(2);
    });

    it('renders accounts with correct address', () => {
      expect(identities.get(0).props.address).to.equal(ACCOUNTS[0]);
    });
  });
});
