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

import Page from './';

const BUTTONS = ['buttonA', 'buttonB'];
const CLASSNAME = 'testClass';
const TESTTEXT = 'testing children';
const TITLE = 'test title';

let component;

function render () {
  component = shallow(
    <Page
      buttons={ BUTTONS }
      className={ CLASSNAME }
      title={ TITLE }
    >
      <div id='testContent'>
        { TESTTEXT }
      </div>
    </Page>
  );

  return component;
}

describe('ui/Page', () => {
  beforeEach(() => {
    render();
  });

  it('renders defaults', () => {
    expect(component).to.be.ok;
  });

  it('renders the children', () => {
    expect(component.find('div[id="testContent"]').text()).to.equal(TESTTEXT);
  });

  describe('components', () => {
    describe('ActionBar', () => {
      let actions;

      beforeEach(() => {
        actions = component.find('Actionbar');
      });

      it('renders the actionbar', () => {
        expect(actions.get(0)).to.be.ok;
      });

      it('passes the provided title', () => {
        expect(actions.props().title).to.equal(TITLE);
      });

      it('passed the provided buttons', () => {
        expect(actions.props().buttons).to.equal(BUTTONS);
      });
    });
  });
});
