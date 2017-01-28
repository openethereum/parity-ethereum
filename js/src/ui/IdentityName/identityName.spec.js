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

import sinon from 'sinon';

import IdentityName from './identityName';

const ADDR_A = '0x123456789abcdef0123456789A';
const ADDR_B = '0x123456789abcdef0123456789B';
const ADDR_C = '0x123456789abcdef0123456789C';
const ADDR_NULL = '0x0000000000000000000000000000000000000000';
const NAME_JIMMY = 'Jimmy Test';
const STORE = {
  dispatch: sinon.stub(),
  subscribe: sinon.stub(),
  getState: () => {
    return {
      balances: {
        tokens: {}
      },
      personal: {
        accountsInfo: {
          [ADDR_A]: { name: 'testing' },
          [ADDR_B]: {}
        }
      }
    };
  }
};

function render (props) {
  return shallow(
    <IdentityName
      store={ STORE }
      { ...props }
    />
  ).find('IdentityName').shallow();
}

describe('ui/IdentityName', () => {
  it('renders defaults', () => {
    expect(render({ address: ADDR_A })).to.be.ok;
  });

  describe('account not found', () => {
    it('renders null with empty', () => {
      expect(
        render({ address: ADDR_C, empty: true }).html()
      ).to.be.null;
    });

    it('renders address without empty', () => {
      expect(
        render({ address: ADDR_C }).text()
      ).to.equal(ADDR_C);
    });

    it('renders short address with shorten', () => {
      expect(
        render({ address: ADDR_C, shorten: true }).find('ShortenedHash').props().data
      ).to.equal(ADDR_C);
    });

    it('renders unknown with flag', () => {
      expect(
        render({ address: ADDR_C, unknown: true }).find('FormattedMessage').props().id
      ).to.equal('ui.identityName.unnamed');
    });

    it('renders name when not found and passed', () => {
      expect(
        render({ address: ADDR_C, name: NAME_JIMMY }).text()
      ).to.equal(NAME_JIMMY.toUpperCase());
    });

    it('renders name when not found, unknown and passed', () => {
      expect(
        render({ address: ADDR_C, name: NAME_JIMMY, unknown: true }).text()
      ).to.equal(NAME_JIMMY.toUpperCase());
    });

    it('renders 0x000...000 as null', () => {
      expect(
        render({ address: ADDR_NULL }).find('FormattedMessage').props().id
      ).to.equal('ui.identityName.null');
    });
  });
});
