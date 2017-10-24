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

import defaults, { MODES } from './defaults';

import Features from './';

let component;
let instance;

function render (props = { visible: true }) {
  component = shallow(
    <Features { ...props } />
  );
  instance = component.instance();

  return component;
}

describe('views/Settings/Features', () => {
  beforeEach(() => {
    render();
  });

  it('renders defaults', () => {
    expect(component).to.be.ok;
  });

  describe('visibility', () => {
    let oldEnv;

    beforeEach(() => {
      oldEnv = process.env.NODE_ENV;
    });

    afterEach(() => {
      process.env.NODE_ENV = oldEnv;
    });

    it('renders null when NODE_ENV === production', () => {
      process.env.NODE_ENV = 'production';
      render();
      expect(component.get(0)).to.be.null;
    });

    it('renders component when NODE_ENV !== production', () => {
      process.env.NODE_ENV = 'development';
      render();
      expect(component.get(0)).not.to.be.null;
    });
  });

  describe('instance methods', () => {
    describe('renderItem', () => {
      const keys = Object.keys(defaults).filter((key) => defaults[key].mode !== MODES.PRODUCTION);
      const key = keys[0];

      let item;

      beforeEach(() => {
        item = instance.renderItem(key);
      });

      it('renders an item', () => {
        expect(item).not.to.be.null;
      });

      it('displays the correct name', () => {
        expect(item.props.primaryText).to.equal(defaults[key].name);
      });
    });
  });
});
