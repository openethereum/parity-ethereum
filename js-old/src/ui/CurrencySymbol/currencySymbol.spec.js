// Copyright 2015-2018 Parity Technologies (UK) Ltd.
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

import CurrencySymbol from './';

let component;
let store;

function createRedux (netChain = 'ropsten') {
  store = {
    dispatch: sinon.stub(),
    subscribe: sinon.stub(),
    getState: () => {
      return {
        nodeStatus: {
          netChain
        }
      };
    }
  };

  return store;
}

function render (netChain, props = {}) {
  component = shallow(
    <CurrencySymbol { ...props } />,
    {
      context: {
        store: createRedux(netChain)
      }
    }
  ).find('CurrencySymbol').shallow();

  return component;
}

describe('ui/CurrencySymbol', () => {
  it('renders defaults', () => {
    expect(render()).to.be.ok;
  });

  it('passes the className as provided', () => {
    expect(render('ropsten', { className: 'test' }).find('span').hasClass('test')).to.be.true;
  });

  describe('currencies', () => {
    it('renders ETH as default', () => {
      expect(render().text()).equal('ETH');
    });

    it('renders ETC for classic', () => {
      expect(render('classic').text()).equal('ETC');
    });

    it('renders ETH as default', () => {
      expect(render('somethingElse').text()).equal('ETH');
    });
  });

  describe('renderSymbol', () => {
    it('render defaults', () => {
      expect(render().instance().renderSymbol()).to.be.ok;
    });

    it('render ETH as default', () => {
      expect(render().instance().renderSymbol()).equal('ETH');
    });

    it('render ETC', () => {
      expect(render('classic').instance().renderSymbol()).equal('ETC');
    });
  });
});
