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

import AccountCard from './';

const TEST_ADDRESS = '0x1234567890123456789012345678901234567890';
const TEST_NAME = 'Jimmy';

let component;
let onClick;
let onFocus;

function render (props = {}) {
  if (!props.account) {
    props.account = {
      address: TEST_ADDRESS,
      description: 'testDescription',
      name: TEST_NAME,
      meta: {
        tags: [ 'tag 1', 'tag 2' ]
      }
    };
  }

  onClick = sinon.stub();
  onFocus = sinon.stub();

  component = shallow(
    <AccountCard
      { ...props }
      onClick={ onClick }
      onFocus={ onFocus }
    />
  );

  return component;
}

describe('ui/AccountCard', () => {
  beforeEach(() => {
    render();
  });

  it('renders defaults', () => {
    expect(component).to.be.ok;
  });

  describe('components', () => {
    describe('Balance', () => {
      let balance;

      beforeEach(() => {
        balance = component.find('Balance');
      });

      it('renders the balance', () => {
        expect(balance.length).to.equal(1);
      });

      it('sets showOnlyEth & showZeroValues', () => {
        expect(balance.props().showOnlyEth).to.be.true;
        expect(balance.props().showZeroValues).to.be.true;
      });
    });

    describe('IdentityIcon', () => {
      let icon;

      beforeEach(() => {
        icon = component.find('Connect(IdentityIcon)');
      });

      it('renders the icon', () => {
        expect(icon.length).to.equal(1);
      });

      it('passes the address through', () => {
        expect(icon.props().address).to.equal(TEST_ADDRESS);
      });
    });

    describe('IdentityName', () => {
      let name;

      beforeEach(() => {
        name = component.find('Connect(IdentityName)');
      });

      it('renders the name', () => {
        expect(name.length).to.equal(1);
      });

      it('passes the address through', () => {
        expect(name.props().address).to.equal(TEST_ADDRESS);
      });

      it('passes the name through', () => {
        expect(name.props().name).to.equal(TEST_NAME);
      });

      it('renders unknown (no name)', () => {
        expect(name.props().unknown).to.be.true;
      });
    });

    describe('Tags', () => {
      let tags;

      beforeEach(() => {
        tags = component.find('Tags');
      });

      it('renders the tags', () => {
        expect(tags.length).to.equal(1);
      });
    });
  });
});
