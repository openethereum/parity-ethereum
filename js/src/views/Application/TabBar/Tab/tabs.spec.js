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

import Tab from './';

let component;
let instance;

function render (id = 'signer') {
  component = shallow(
    <Tab
      pending={ 5 }
      view={ { id } }
    />
  );
  instance = component.instance();

  return component;
}

describe('views/Application/TabBar/Tab', () => {
  beforeEach(() => {
    render();
  });

  it('renders defaults', () => {
    expect(component).to.be.ok;
  });

  describe('instance methods', () => {
    describe('renderLabel', () => {
      it('renders the label with correct label', () => {
        expect(
          shallow(instance.renderLabel('test')).find('FormattedMessage').props().id
        ).to.equal('settings.views.test.label');
      });

      it('renders the bubble when passed', () => {
        expect(
          shallow(instance.renderLabel('test', 'testBubble')).text()
        ).to.equal('<FormattedMessage />testBubble');
      });
    });

    describe('renderSignerLabel', () => {
      beforeEach(() => {
        sinon.stub(instance, 'renderLabel');
      });

      afterEach(() => {
        instance.renderLabel.restore();
      });

      it('calls renderLabel with the details', () => {
        instance.renderSignerLabel();
        expect(instance.renderLabel).to.have.been.calledWith('signer');
      });
    });
  });
});
