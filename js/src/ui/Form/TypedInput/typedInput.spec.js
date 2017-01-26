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

import { ABI_TYPES } from '~/util/abi';

import TypedInput from './';

let component;
let select;
let onChange;

function render (props) {
  onChange = sinon.stub();
  component = shallow(
    <TypedInput
      { ...props }
      onChange={ onChange }
    />
  );
  select = component.find('Select');

  return component;
}

describe('ui/Form/TypedInput', () => {
  describe('bool selection', () => {
    beforeEach(() => {
      render({ param: { type: ABI_TYPES.BOOL } });
    });

    it('renders', () => {
      expect(component).to.be.ok;
    });

    it('calls onChange when value changes', () => {
      select.shallow().simulate('change', { target: { value: 'true' } });

      expect(onChange).to.have.been.called;
    });

    it("calls onChange(true) when value changes to 'true'", () => {
      select.shallow().simulate('change', { target: { value: 'true' } });

      expect(onChange).to.have.been.calledWith(true);
    });

    it("calls onChange(false) when value changes to 'false'", () => {
      select.shallow().simulate('change', { target: { value: 'false' } });

      expect(onChange).to.have.been.calledWith(false);
    });
  });
});
