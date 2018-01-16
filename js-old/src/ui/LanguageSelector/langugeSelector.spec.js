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

import { LocaleStore } from '~/i18n';

import LanguageSelector from './';

let component;

function render (props = {}) {
  component = shallow(
    <LanguageSelector { ...props } />
  );

  return component;
}

describe('LanguageSelector', () => {
  it('renders defaults', () => {
    expect(render()).to.be.ok;
  });

  describe('Select', () => {
    let select;
    let localeStore;

    beforeEach(() => {
      localeStore = LocaleStore.get();
      sinon.stub(localeStore, 'setLocale');

      render();
      select = component.find('Select');
    });

    afterEach(() => {
      localeStore.setLocale.restore();
    });

    it('renders the Select', () => {
      expect(select).to.have.length(1);
    });

    it('has locale items', () => {
      expect(select.find('MenuItem').length > 0).to.be.true;
    });

    it('calls localeStore.setLocale when changed', () => {
      select.simulate('change', { target: { value: 'de' } });
      expect(localeStore.setLocale).to.have.been.calledWith('de');
    });
  });
});
