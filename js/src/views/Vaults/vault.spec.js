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

import { createApi } from './vaults.test.js';

import Vaults from './';

let api;
let component;
let instance;

function render (props = {}) {
  api = createApi();

  component = shallow(
    <Vaults />,
    {
      context: { api }
    }
  );
  instance = component.instance();

  return component;
}

describe('modals/Vaults', () => {
  beforeEach(() => {
    render();
  });

  it('renders defaults', () => {
    expect(component).to.be.ok;
  });

  describe('Portal opened state', () => {
    let portal;

    beforeEach(() => {
      instance.store.setOpen(true);
      portal = component.find('Portal');
    });

    it('renders null when not opened', () => {
      instance.store.setOpen(false);
      expect(component.get(0)).to.be.null;
    });

    it('renders Portal when opened', () => {
      expect(portal).not.to.be.null;
    });

    it('contains onClose from instance', () => {
      expect(portal.props().onClose).to.equal(instance.onClose);
    });

    it('contains the correct title', () => {
      expect(portal.props().title.props.id).to.equal('vaults.title');
    });
  });

  describe('instance methods', () => {
    describe('onClose', () => {
      beforeEach(() => {
        sinon.spy(instance.store, 'closeModal');
      });

      afterEach(() => {
        instance.store.closeModal.restore();
      });

      it('calls into store.closeModal', () => {
        instance.onClose();
        expect(instance.store.closeModal).to.have.been.called;
      });
    });
  });
});
