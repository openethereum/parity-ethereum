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

import React, { Component } from 'react';

import PlaygroundExample from '~/playground/playgroundExample';

import ConnectedCurrencySymbol, { CurrencySymbol } from './currencySymbol';

export default class CurrencySymbolExample extends Component {
  render () {
    return (
      <div>
        <PlaygroundExample name='Connected Currency Symbol'>
          <ConnectedCurrencySymbol />
        </PlaygroundExample>

        <PlaygroundExample name='Simple Currency Symbol'>
          <CurrencySymbol
            netChain='testnet'
          />
        </PlaygroundExample>

        <PlaygroundExample name='ETC Currency Symbol'>
          <CurrencySymbol
            netChain='classic'
          />
        </PlaygroundExample>

        <PlaygroundExample name='EXP Currency Symbol'>
          <CurrencySymbol
            netChain='expanse'
          />
        </PlaygroundExample>
      </div>
    );
  }
}
