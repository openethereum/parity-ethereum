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

import SelectionList from './';

const ITEMS = ['A', 'B', 'C'];

let component;
let instance;
let renderItem;
let onDefaultClick;
let onSelectClick;

function render (props = {}) {
  renderItem = sinon.stub();
  onDefaultClick = sinon.stub();
  onSelectClick = sinon.stub();

  component = shallow(
    <SelectionList
      items={ ITEMS }
      noStretch='testNoStretch'
      onDefaultClick={ onDefaultClick }
      onSelectClick={ onSelectClick }
      renderItem={ renderItem }
    />
  );
  instance = component.instance();

  return component;
}

describe('ui/SelectionList', () => {
  beforeEach(() => {
    render();
  });

  it('renders defaults', () => {
    expect(component).to.be.ok;
  });

  describe('SectionList', () => {
    let section;

    beforeEach(() => {
      section = component.find('SectionList');
    });

    it('renders the SectionList', () => {
      expect(section.get(0)).to.be.ok;
    });

    it('passes the items through', () => {
      expect(section.props().items).to.deep.equal(ITEMS);
    });

    it('passes internal render method', () => {
      expect(section.props().renderItem).to.equal(instance.renderItem);
    });

    it('passes noStretch prop through', () => {
      expect(section.props().noStretch).to.equal('testNoStretch');
    });
  });

  describe('instance methods', () => {
    describe('renderItem', () => {
      let result;

      beforeEach(() => {
        result = instance.renderItem('testItem', 'testIndex');
      });

      it('renders', () => {
        expect(result).to.be.ok;
      });

      it('calls into parent renderItem', () => {
        expect(renderItem).to.have.been.calledWith('testItem', 'testIndex');
      });
    });
  });
});
