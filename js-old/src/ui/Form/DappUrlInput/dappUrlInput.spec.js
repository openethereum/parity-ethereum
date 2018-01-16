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

import DappUrlInput from './';

let component;
let onChange;
let onGoto;
let onRestore;

function render (props = { url: 'http://some.url' }) {
  onChange = sinon.stub();
  onGoto = sinon.stub();
  onRestore = sinon.stub();

  component = shallow(
    <DappUrlInput
      onChange={ onChange }
      onGoto={ onGoto }
      onRestore={ onRestore }
      { ...props }
    />
  );

  return component;
}

describe('ui/Form/DappUrlInput', () => {
  it('renders defaults', () => {
    expect(render()).to.be.ok;
  });

  describe('events', () => {
    describe('onChange', () => {
      it('calls the onChange callback as provided', () => {
        component.simulate('change', { target: { value: 'testing' } });
        expect(onChange).to.have.been.calledWith('testing');
      });
    });

    describe('onKeyDown', () => {
      it('calls the onGoto callback on enter', () => {
        component.simulate('keyDown', { keyCode: 13 });
        expect(onGoto).to.have.been.called;
      });

      it('calls the onRestor callback on esc', () => {
        component.simulate('keyDown', { keyCode: 27 });
        expect(onRestore).to.have.been.called;
      });
    });
  });
});
