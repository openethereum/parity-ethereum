// Copyright 2015, 2016 Parity Technologies (UK) Ltd.
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

import ParityBar from './parityBar';

let component;
let instance;
let store;

function createRedux (state = {}) {
  store = {
    dispatch: sinon.stub(),
    subscribe: sinon.stub(),
    getState: () => Object.assign({ signer: { pending: [] } }, state)
  };

  return store;
}

function render (props = {}, state = {}) {
  component = shallow(
    <ParityBar { ...props } />,
    { context: { store: createRedux(state) } }
  ).find('ParityBar').shallow();
  instance = component.instance();

  return component;
}

describe.only('views/ParityBar', () => {
  it('renders defaults', () => {
    expect(render()).to.be.ok;
  });

  describe('renderLabel', () => {
    beforeEach(() => {
      render();
    });

    it('renders the label name', () => {
      expect(shallow(instance.renderLabel('testing', null)).text()).to.equal('testing');
    });

    it('renders name and bubble', () => {
      expect(shallow(instance.renderLabel('testing', '(bubble)')).text()).to.equal('testing(bubble)');
    });
  });

  describe('renderSignerLabel', () => {
    beforeEach(() => {
      render();
    });

    it('renders the signer label', () => {
      expect(shallow(instance.renderSignerLabel()).find('FormattedMessage').props().id).to.equal('parityBar.label.parity');
    });

    it('does not render a badge when no pendings', () => {
      expect(shallow(instance.renderSignerLabel()).find('Badge')).to.have.length(0);
    });

    it('renders a badge when with pendings', () => {
      render({}, { signer: { pending: ['123', '456'] } });
      expect(shallow(instance.renderSignerLabel()).find('Badge').props().value).to.equal(2);
    });
  });
});
