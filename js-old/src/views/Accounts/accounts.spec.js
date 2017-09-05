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

let api;
let component;
let hwstore;
let instance;
let redux;

function createApi () {
  api = {};

  return api;
}

function createHwStore (walletAddress = '0x456') {
  hwstore = {
    wallets: {
      [walletAddress]: {
        address: walletAddress
      }
    },
    createAccountInfo: sinon.stub()
  };

  return hwstore;
}

function createRedux () {
  redux = {
    dispatch: sinon.stub(),
    subscribe: sinon.stub(),
    getState: () => {
      return {
        personal: {
          accounts: {},
          accountsInfo: {
            '0x123': { meta: '1' },
            '0x999': { meta: { hardware: {} } }
          }
        },
        balances: {
          balances: {}
        },
        nodeStatus: {
          nodeKind: {
            'availability': 'personal'
          }
        }
      };
    }
  };

  return redux;
}

function render (props = {}) {
  component = shallow(
    <Accounts { ...props } />,
    {
      context: {
        store: createRedux()
      }
    }
  ).find('Accounts').shallow({
    context: {
      api: createApi()
    }
  });
  instance = component.instance();

  return component;
}

describe('views/Accounts', () => {
  beforeEach(() => {
    render();
  });

  it('renders defaults', () => {
    expect(component).to.be.ok;
  });

  describe('instance event methods', () => {
    describe('onHardwareChange', () => {
      it('detects completely new entries', () => {
        instance.hwstore = createHwStore();
        instance.onHardwareChange();

        expect(hwstore.createAccountInfo).to.have.been.calledWith({ address: '0x456' });
      });

      it('detects addressbook entries', () => {
        instance.hwstore = createHwStore('0x123');
        instance.onHardwareChange();

        expect(hwstore.createAccountInfo).to.have.been.calledWith({ address: '0x123' }, { meta: '1' });
      });

      it('ignores existing hardware entries', () => {
        instance.hwstore = createHwStore('0x999');
        instance.onHardwareChange();

        expect(hwstore.createAccountInfo).not.to.have.been.called;
      });
    });
  });
});
