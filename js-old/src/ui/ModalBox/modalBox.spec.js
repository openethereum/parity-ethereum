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

import ModalBox from './';

let component;

const CHILDREN = <div id='testChild'>testChild</div>;
const ICON = <div id='testIcon'>testIcon</div>;
const SUMMARY = <div id='testSummary'>testSummary</div>;

function render () {
  component = shallow(
    <ModalBox
      children={ CHILDREN }
      icon={ ICON }
      summary={ SUMMARY }
    />
  );

  return component;
}

describe('ui/ModalBox', () => {
  beforeEach(() => {
    render();
  });

  it('renders defaults', () => {
    expect(component).to.be.ok;
  });

  it('adds the icon as supplied', () => {
    expect(component.find('#testIcon').text()).to.equal('testIcon');
  });

  describe('components', () => {
    it('adds the Body as supplied', () => {
      expect(component.find('Body').props().children).to.deep.equal(CHILDREN);
    });

    it('adds the Summary as supplied', () => {
      expect(component.find('Summary').props().summary).to.deep.equal(SUMMARY);
    });
  });
});
