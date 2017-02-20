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

import VaultCard from './';

const VAULT = { name: 'testing', isOpen: true };

let component;
let instance;

function render (props = {}) {
  component = shallow(
    <VaultCard
      vault={ VAULT }
      { ...props }
    />
  );
  instance = component.instance();

  return component;
}

describe('ui/VaultCard', () => {
  beforeEach(() => {
    render();
  });

  it('renders defaults', () => {
    expect(component).to.be.ok;
  });

  describe('components', () => {
    describe('Layout', () => {
      let layout;

      beforeEach(() => {
        layout = component.find('Layout');
      });

      it('renders', () => {
        expect(layout.get(0)).to.be.ok;
      });

      it('passes the vault', () => {
        expect(layout.props().vault).to.deep.equal(VAULT);
      });
    });
  });

  describe('instance methods', () => {
    describe('renderAccounts', () => {
      it('renders empty when no accounts supplied', () => {
        expect(
          shallow(instance.renderAccounts()).find('FormattedMessage').props().id
        ).to.equal('vaults.accounts.empty');
      });

      describe('with accounts', () => {
        const ACCOUNTS = ['0x123', '0x456'];
        let identities;

        beforeEach(() => {
          render({ accounts: ACCOUNTS });
          identities = shallow(instance.renderAccounts()).find('Connect(IdentityIcon)');
        });

        it('renders the accounts when supplied', () => {
          expect(identities).to.have.length(2);
        });

        it('renders accounts with correct address', () => {
          console.log(identities.get(0));
          expect(identities.get(0).props.address).to.equal(ACCOUNTS[0]);
        });
      });
    });
  });
});
