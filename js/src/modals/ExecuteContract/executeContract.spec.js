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

import ExecuteContract from './';

import { createApi, CONTRACT, STORE } from './executeContract.test.js';

let component;
let onClose;
let onFromAddressChange;

function render (props) {
  onClose = sinon.stub();
  onFromAddressChange = sinon.stub();

  component = shallow(
    <ExecuteContract
      { ...props }
      contract={ CONTRACT }
      onClose={ onClose }
      onFromAddressChange={ onFromAddressChange }
    />,
      { context: { api: createApi(), store: STORE } }
  ).find('ExecuteContract').shallow();

  return component;
}

describe('modals/ExecuteContract', () => {
  it('renders', () => {
    expect(render({ accounts: {} })).to.be.ok;
  });

  describe('instance functions', () => {
    beforeEach(() => {
      render({
        accounts: {}
      });
    });

    describe('onValueChange', () => {
      it('toggles boolean from false to true', () => {
        component.setState({
          func: CONTRACT.functions[0],
          values: [false]
        });
        component.instance().onValueChange(null, 0, true);

        expect(component.state().values).to.deep.equal([true]);
      });
    });
  });
});
