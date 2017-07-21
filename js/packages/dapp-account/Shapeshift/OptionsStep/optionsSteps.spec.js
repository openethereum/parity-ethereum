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

import Store, { WARNING_NO_PRICE } from '../store';

import OptionsStep from './';

const ADDRESS = '0x1234567890123456789012345678901234567890';

let component;
let instance;
let store;

function render () {
  store = new Store(ADDRESS);
  component = shallow(
    <OptionsStep store={ store } />
  );
  instance = component.instance();

  return component;
}

describe('views/Account/Shapeshift/OptionsStep', () => {
  beforeEach(() => {
    render();
  });

  it('renders defaults', () => {
    expect(component).to.be.ok;
  });

  it('renders no coins when none available', () => {
    expect(component.find('FormattedMessage').props().id).to.equal('shapeshift.optionsStep.noPairs');
  });

  describe('components', () => {
    beforeEach(() => {
      store.setCoins([{ symbol: 'BTC', name: 'Bitcoin' }]);
      store.toggleAcceptTerms();
    });

    describe('terms Checkbox', () => {
      it('shows the state of store.hasAcceptedTerms', () => {
        expect(component.find('Checkbox').props().checked).to.be.true;
      });
    });

    describe('warning', () => {
      let warning;

      beforeEach(() => {
        store.setWarning(WARNING_NO_PRICE);
        warning = component.find('Warning');
      });

      it('shows a warning message when available', () => {
        expect(warning.props().warning.props.id).to.equal('shapeshift.warning.noPrice');
      });
    });
  });

  describe('events', () => {
    describe('onChangeRefundAddress', () => {
      beforeEach(() => {
        sinon.stub(store, 'setRefundAddress');
      });

      afterEach(() => {
        store.setRefundAddress.restore();
      });

      it('sets the refundAddress on the store', () => {
        instance.onChangeRefundAddress(null, 'refundAddress');
        expect(store.setRefundAddress).to.have.been.calledWith('refundAddress');
      });
    });

    describe('onSelectCoin', () => {
      beforeEach(() => {
        sinon.stub(store, 'setCoinSymbol');
      });

      afterEach(() => {
        store.setCoinSymbol.restore();
      });

      it('sets the coinSymbol on the store', () => {
        instance.onSelectCoin(null, 'XMR');
        expect(store.setCoinSymbol).to.have.been.calledWith('XMR');
      });
    });

    describe('onToggleAcceptTerms', () => {
      beforeEach(() => {
        sinon.stub(store, 'toggleAcceptTerms');
      });

      afterEach(() => {
        store.toggleAcceptTerms.restore();
      });

      it('toggles the terms on the store', () => {
        instance.onToggleAcceptTerms();
        expect(store.toggleAcceptTerms).to.have.been.called;
      });
    });
  });
});
