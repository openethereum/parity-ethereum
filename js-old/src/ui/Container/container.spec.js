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

import Container from './container';

function render (props) {
  return shallow(
    <Container { ...props } />
  );
}

describe('ui/Container', () => {
  describe('rendering', () => {
    it('renders defaults', () => {
      expect(render()).to.be.ok;
    });

    it('renders with the specified className', () => {
      expect(render({ className: 'testClass' })).to.have.className('testClass');
    });
  });

  describe('sections', () => {
    it('renders the default Card', () => {
      expect(render().find('Card')).to.have.length(1);
    });

    it('renders Hover Card when available', () => {
      const cards = render({ hover: <div>testingHover</div> }).find('Card');

      expect(cards).to.have.length(2);
      expect(cards.get(1).props.children.props.children).to.equal('testingHover');
    });

    it('renders the Title', () => {
      const title = render({ title: 'title' }).find('Title');

      expect(title).to.have.length(1);
      expect(title.props().title).to.equal('title');
    });
  });
});
