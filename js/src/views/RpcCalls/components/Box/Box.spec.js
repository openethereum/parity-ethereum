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

import React from 'react';
import { shallow } from 'enzyme';

import Box from './Box';

describe('views/Status/components/Box', () => {
  describe('rendering', () => {
    const title = 'test title';
    let rendered;

    beforeEach(() => {
      rendered = shallow(<Box title={ title } />);
    });

    it('renders the component', () => {
      expect(rendered).to.be.ok;
      expect(rendered).to.have.className('dapp-box');
    });

    it('renders the title', () => {
      expect(rendered.find('h2')).to.have.text(title);
    });

    it('renders no default value', () => {
      expect(rendered).to.not.have.descendants('h1');
    });
  });

  describe('contents', () => {
    const value = 'test value';
    const child = 'this is the child value';

    let rendered;

    beforeEach(() => {
      rendered = shallow(
        <Box
          title='title'
          value={ value }
        >
          <pre>{ child }</pre>
        </Box>
      );
    });

    it('renders the value', () => {
      expect(rendered.find('h1')).to.have.text(value);
    });

    it('wraps the children', () => {
      expect(rendered.find('pre')).to.have.text(child);
    });
  });
});
