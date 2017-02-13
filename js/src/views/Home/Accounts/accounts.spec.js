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

import Accounts from './';

let component;
let store;

function createRedux () {
  store = {
    dispatch: sinon.stub(),
    subscribe: sinon.stub(),
    getState: () => {
      return {
        personal: {
          accountsInfo: { '0x123': {} }
        }
      };
    }
  };

  return store;
}

function render (history = []) {
  component = shallow(
    <Accounts
      history={ history }
    />,
    {
      context: {
        store: createRedux()
      }
    }
  ).find('Accounts').shallow();

  return component;
}

describe('views/Home/Accounts', () => {
  it('renders defaults', () => {
    expect(render()).to.be.ok;
  });

  describe('no history', () => {
    beforeEach(() => {
      render();
    });

    it('renders empty message', () => {
      expect(component.find('FormattedMessage').props().id).to.equal('home.accounts.none');
    });
  });
});
