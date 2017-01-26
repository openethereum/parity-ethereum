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

import Tags from './';

const TEST_TAGS = ['tagA', 'tagB'];

let component;
let handleAddSearchToken;
let instance;

function render (tags = TEST_TAGS) {
  handleAddSearchToken = sinon.stub();
  component = shallow(
    <Tags
      className='testClass'
      handleAddSearchToken={ handleAddSearchToken }
      tags={ tags }
    />
  );
  instance = component.instance();

  return component;
}

describe('Tags', () => {
  beforeEach(() => {
    render();
  });

  it('renders defaults', () => {
    expect(component).to.be.ok;
  });

  it('renders null with no tokens', () => {
    expect(render([]).get(0)).to.be.null;
  });

  it('adds the className as provided', () => {
    expect(component.hasClass('testClass')).to.be.true;
  });

  describe('renderTags', () => {
    let tags;

    beforeEach(() => {
      tags = instance.renderTags();
    });

    it('renders each of the tags', () => {
      expect(tags.length).to.equal(2);
      expect(tags[0].props.children).to.equal(TEST_TAGS[0]);
      expect(tags[1].props.children).to.equal(TEST_TAGS[1]);
    });

    it('adds key from index', () => {
      expect(tags[0].key).to.equal('0');
      expect(tags[1].key).to.equal('1');
    });

    describe('onClick', () => {
      let tag;

      beforeEach(() => {
        tag = shallow(tags[1]);
        tag.simulate('click', {});
      });

      it('calls handleAddSearchToken', () => {
        expect(handleAddSearchToken).to.have.been.calledWith(TEST_TAGS[1]);
      });
    });
  });
});
