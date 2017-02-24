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

import DappPermissions from './';

let component;
let store;

function createRedux () {
  store = {
    dispatch: sinon.stub(),
    subscribe: sinon.stub(),
    getState: () => {
      return {
        balances: {
          balances: {}
        }
      };
    }
  };

  return store;
}

function renderShallow (permissionStore = {}) {
  component = shallow(
    <DappPermissions permissionStore={ permissionStore } />,
    {
      context: {
        store: createRedux()
      }
    }
  ).find('DappPermissions').shallow();

  return component;
}

describe('modals/DappPermissions', () => {
  describe('rendering', () => {
    it('renders defaults', () => {
      expect(renderShallow()).to.be.ok;
    });

    it('does not render the modal with modalOpen = false', () => {
      expect(
        renderShallow({ modalOpen: false }).find('Portal')
      ).to.have.length(0);
    });

    it('does render the modal with modalOpen = true', () => {
      expect(
        renderShallow({ modalOpen: true, accounts: [] }).find('Portal')
      ).to.have.length(1);
    });
  });
});
