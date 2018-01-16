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

import ParityBar from './';

import { createApi } from './parityBar.test.js';

let api;
let component;
let instance;
let store;

function createRedux (state = {}) {
  store = {
    dispatch: sinon.stub(),
    subscribe: sinon.stub(),
    getState: () => Object.assign({
      balances: {
        balances: {}
      },
      signer: {
        pending: []
      },
      nodeStatus: {
        health: {
          overall: {
            status: 'ok',
            message: []
          }
        }
      }
    }, state)
  };

  return store;
}

function render (props = {}, state = {}) {
  api = createApi();
  component = shallow(
    <ParityBar { ...props } />,
    {
      context: {
        store: createRedux(state)
      }
    }
  ).find('ParityBar').shallow({ context: { api } });
  instance = component.instance();

  return component;
}

describe('ParityBar', () => {
  beforeEach(() => {
    render({ dapp: true });
  });

  it('renders defaults', () => {
    expect(component).to.be.ok;
  });

  describe('renderBar', () => {
    let bar;

    beforeEach(() => {
      bar = shallow(instance.renderBar());
    });

    it('renders nothing when not overlaying a dapp', () => {
      render({ dapp: false });
      expect(instance.renderBar()).to.be.null;
    });

    it('renders when overlaying a dapp', () => {
      expect(bar.find('div')).not.to.have.length(0);
    });

    it('renders the Parity button', () => {
      const label = shallow(bar.find('Button').at(1).props().label);

      expect(label.find('FormattedMessage').props().id).to.equal('parityBar.label.parity');
    });

    it('renders the Signer button', () => {
      const label = shallow(bar.find('Button').last().props().label);

      expect(label.find('FormattedMessage').props().id).to.equal('parityBar.label.signer');
    });
  });

  describe('renderExpanded', () => {
    let expanded;

    beforeEach(() => {
      expanded = shallow(instance.renderExpanded());
    });

    it('includes the Signer', () => {
      expect(expanded.find('Connect(Embedded)')).to.have.length(1);
    });
  });

  describe('renderLabel', () => {
    it('renders the label name', () => {
      expect(shallow(instance.renderLabel('testing', null)).text()).to.equal('testing');
    });

    it('renders name and bubble', () => {
      expect(shallow(instance.renderLabel('testing', '(bubble)')).text()).to.equal('testing(bubble)');
    });
  });

  describe('renderSignerLabel', () => {
    let label;

    beforeEach(() => {
      label = shallow(instance.renderSignerLabel());
    });

    it('renders the signer label', () => {
      expect(label.find('FormattedMessage').props().id).to.equal('parityBar.label.signer');
    });

    it('renders a badge when pending requests', () => {
      render({}, { signer: { pending: ['123', '456'] } });
      expect(shallow(instance.renderSignerLabel()).find('SignerPending')).to.be.ok;
    });
  });

  describe('opened state', () => {
    beforeEach(() => {
      sinon.spy(instance, 'renderBar');
      sinon.spy(instance, 'renderExpanded');
    });

    afterEach(() => {
      instance.renderBar.restore();
      instance.renderExpanded.restore();
    });

    it('renders expanded with opened === true', () => {
      expect(instance.renderExpanded).not.to.have.been.called;
      instance.store.setOpen(true);
      expect(instance.renderExpanded).to.have.been.called;
    });
  });
});
