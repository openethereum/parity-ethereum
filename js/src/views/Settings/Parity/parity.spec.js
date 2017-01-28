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

import { createApi } from './parity.test.js';
import Parity from './';

let component;
let instance;

function render (props = {}) {
  component = shallow(
    <Parity { ...props } />,
    { context: { api: createApi() } }
  );
  instance = component.instance();

  return component;
}

describe('views/Settings/Parity', () => {
  beforeEach(() => {
    render();
    sinon.spy(instance.store, 'loadMode');
  });

  afterEach(() => {
    instance.store.loadMode.restore();
  });

  it('renders defaults', () => {
    expect(component).to.be.ok;
  });

  describe('componentWillMount', () => {
    beforeEach(() => {
      return instance.componentWillMount();
    });

    it('loads the mode in the store', () => {
      expect(instance.store.loadMode).to.have.been.called;
    });
  });

  describe('components', () => {
    it('renders a Container component', () => {
      expect(component.find('Container')).to.have.length(1);
    });

    it('renders a LanguageSelector component', () => {
      expect(component.find('LanguageSelector')).to.have.length(1);
    });

    it('renders a Features component', () => {
      expect(component.find('LanguageSelector')).to.have.length(1);
    });
  });

  describe('Parity features', () => {
    describe('mode selector', () => {
      let select;

      beforeEach(() => {
        select = component.find('Select[id="parityModeSelect"]');
        sinon.spy(instance.store, 'changeMode');
      });

      afterEach(() => {
        instance.store.changeMode.restore();
      });

      it('renders a mode selector', () => {
        expect(select).to.have.length(1);
      });

      it('changes the mode on the store when changed', () => {
        select.simulate('change', { target: { value: 'dark' } });
        expect(instance.store.changeMode).to.have.been.calledWith('dark');
      });
    });
  });
});
