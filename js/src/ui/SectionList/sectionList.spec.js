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

import SectionList from './';

const ITEMS = ['itemA', 'itemB', 'itemC', 'itemD', 'itemE'];

let component;
let instance;
let renderItem;

function render (props = {}) {
  renderItem = sinon.stub().returns('someThing');
  component = shallow(
    <SectionList
      className='testClass'
      items={ ITEMS }
      renderItem={ renderItem }
      section='testSection'
    />
  );
  instance = component.instance();

  return component;
}

describe('SectionList', () => {
  beforeEach(() => {
    render();
  });

  it('renders defaults', () => {
    expect(component).to.be.ok;
  });

  it('adds className as specified', () => {
    expect(component.hasClass('testClass')).to.be.true;
  });

  describe('instance methods', () => {
    describe('renderRow', () => {
      let row;

      beforeEach(() => {
        sinon.stub(instance, 'renderItem');
        row = instance.renderRow(['testA', 'testB']);
      });

      afterEach(() => {
        instance.renderItem.restore();
      });

      it('renders a row', () => {
        expect(row).to.be.ok;
      });

      it('adds a key for the row', () => {
        expect(row.key).to.be.ok;
      });
    });

    describe('renderItem', () => {
      let item;

      beforeEach(() => {
        item = instance.renderItem('testItem', 50);
      });

      it('renders an item', () => {
        expect(item).to.be.ok;
      });

      it('adds a key for the item', () => {
        expect(item.key).to.be.ok;
      });

      it('calls the external renderer', () => {
        expect(renderItem).to.have.been.calledWith('testItem', 50);
      });
    });
  });
});
