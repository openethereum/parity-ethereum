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

import Home from './';

let api;
let component;

function createApi () {
  api = {};

  return api;
}

function render () {
  component = shallow(
    <Home />,
    {
      context: {
        api: createApi()
      }
    }
  );

  return component;
}

describe('views/Home', () => {
  beforeEach(() => {
    render();
  });

  it('renders defaults', () => {
    expect(component).to.be.ok;
  });

  describe('components', () => {
    it('renders Accounts', () => {
      expect(component.find('Accounts').length).to.equal(1);
    });

    it('renders Dapps', () => {
      expect(component.find('Dapps').length).to.equal(1);
    });

    it('renders News', () => {
      expect(component.find('News').length).to.equal(1);
    });

    it('renders Urls', () => {
      expect(component.find('Urls').length).to.equal(1);
    });
  });
});
