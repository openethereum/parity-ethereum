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

import ExportAccount from './';

const ADDRESS = '0x0123456789012345678901234567890123456789';

let component;
let NEWERROR;
let PAI;
let ONCLOSE;

let reduxStore;

function createReduxStore () {
  reduxStore = {
    dispatch: sinon.stub(),
    subscribe: sinon.stub(),
    getState: () => {
      return {
        balances: {
          balances: {
            [ADDRESS]: {}
          }
        },
        personal: {
          accounts: {
            [ADDRESS]: {
              address: ADDRESS
            }
          }
        }
      };
    }
  };

  return reduxStore;
}

function render () {
  component = shallow(
    <ExportAccount
      newError={ NEWERROR }
      personalAccountsInfo={ PAI }
      onClose={ ONCLOSE }
    />,
    {
      context: { api: {}, store: createReduxStore() }
    }
  );

  return component;
}

describe('CreateExportModal', () => {
  it('renders defaults', () => {
    expect(render()).to.be.ok;
  });
});
