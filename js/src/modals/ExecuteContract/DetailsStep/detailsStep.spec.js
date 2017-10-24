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

import DetailsStep from './';

import { CONTRACT } from '../executeContract.test.js';

let component;
let onAmountChange;
let onClose;
let onFromAddressChange;
let onFuncChange;
let onGasEditClick;
let onValueChange;

function render (props) {
  onAmountChange = sinon.stub();
  onClose = sinon.stub();
  onFromAddressChange = sinon.stub();
  onFuncChange = sinon.stub();
  onGasEditClick = sinon.stub();
  onValueChange = sinon.stub();

  component = shallow(
    <DetailsStep
      { ...props }
      contract={ CONTRACT }
      onAmountChange={ onAmountChange }
      onClose={ onClose }
      onFromAddressChange={ onFromAddressChange }
      onFuncChange={ onFuncChange }
      onGasEditClick={ onGasEditClick }
      onValueChange={ onValueChange }
    />
  );

  return component;
}

describe('modals/ExecuteContract/DetailsStep', () => {
  it('renders', () => {
    expect(render({ accounts: {}, values: [ true ], valuesError: [ null ] })).to.be.ok;
  });

  describe('parameter values', () => {
    beforeEach(() => {
      render({
        accounts: {},
        func: CONTRACT.functions[0],
        values: [ false ],
        valuesError: [ null ]
      });
    });

    describe('bool parameters', () => {
      it('toggles from false to true', () => {
        component.find('TypedInput').last().shallow().simulate('change', { target: { value: 'true' } });

        expect(onValueChange).to.have.been.calledWith(null, 0, true);
      });
    });
  });
});
