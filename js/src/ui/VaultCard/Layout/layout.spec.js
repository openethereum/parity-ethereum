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

import Layout from './';

const DESCRIPTION = 'some description';
const NAME = 'testName';

let component;

function render () {
  component = shallow(
    <Layout
      vault={ {
        isOpen: true,
        meta: {
          description: DESCRIPTION,
          passwordHint: 'some hint'
        },
        name: NAME
      } }
    />
  );

  return component;
}

describe('ui/VaultCard/Layout', () => {
  beforeEach(() => {
    render();
  });

  it('renders defaults', () => {
    expect(component).to.be.ok;
  });

  describe('components', () => {
    describe('IdentityIcon', () => {
      let icon;

      beforeEach(() => {
        icon = component.find('Connect(IdentityIcon)');
      });

      it('renders', () => {
        expect(icon.get(0)).to.be.ok;
      });

      it('passes the name as address key', () => {
        expect(icon.props().address).to.equal(NAME);
      });
    });

    describe('Title', () => {
      let title;

      beforeEach(() => {
        title = component.find('Title');
      });

      it('renders', () => {
        expect(title.get(0)).to.be.ok;
      });

      it('passes the name as title', () => {
        expect(title.props().title).to.equal(NAME);
      });

      it('passes the description as byline', () => {
        expect(title.props().byline).to.equal(DESCRIPTION);
      });
    });
  });
});
