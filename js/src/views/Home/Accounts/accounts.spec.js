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

import Accounts from './';

let component;

function render (history = []) {
  component = shallow(
    <Accounts
      history={ history }
    />
  );

  return component;
}

describe('views/Home/Accounts', () => {
  it('renders defaults', () => {
    expect(render()).to.be.ok;
  });

  describe('no history', () => {
    beforeEach(() => {
      render();
    });

    it('renders empty message', () => {
      expect(component.find('FormattedMessage').props().id).to.equal('home.accounts.none');
    });
  });

  describe('with history', () => {
    const HISTORY = [
      { timestamp: 1, entry: 'testABC' },
      { timestamp: 2, entry: 'testDEF' }
    ];

    beforeEach(() => {
      render(HISTORY);
    });

    it('renders table rows', () => {
      expect(component.find('tr').length).to.equal(HISTORY.length);
    });

    it('renders links', () => {
      expect(component.find('Link').length).to.equal(HISTORY.length);
    });

    it('has links with account id', () => {
      expect(component.find('Link').at(0).props().to).to.equal(`/accounts/${HISTORY[0].entry}`);
    });
  });
});
