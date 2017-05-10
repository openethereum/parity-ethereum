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

import { observer } from 'mobx-react';
import React, { Component } from 'react';

import AccountCard from '~/ui/AccountCard/accountCard.example';
import CurrencySymbol from '~/ui/CurrencySymbol/currencySymbol.example';
import Portal from '~/ui/Portal/portal.example';
import QrCode from '~/ui/QrCode/qrCode.example';
import SectionList from '~/ui/SectionList/sectionList.example';

import PlaygroundStore from './store';
import styles from './playground.css';

PlaygroundStore.register(<AccountCard />);
PlaygroundStore.register(<CurrencySymbol />);
PlaygroundStore.register(<Portal />);
PlaygroundStore.register(<QrCode />);
PlaygroundStore.register(<SectionList />);

@observer
export default class Playground extends Component {
  store = PlaygroundStore.get();

  render () {
    const { component, components } = this.store;

    return (
      <div className={ styles.container }>
        <div className={ styles.title }>
          <span>Playground - </span>
          <select
            className={ styles.select }
            onChange={ this.handleChange }
          >
            {
              components.map((element, index) => {
                const name = element.type.displayName || element.type.name;

                return (
                  <option
                    key={ `${name}_${index}` }
                    value={ index }
                  >
                    { name }
                  </option>
                );
              })
            }
          </select>
        </div>

        <div className={ styles.examples }>
          { component }
        </div>
      </div>
    );
  }

  handleChange = (event) => {
    const { value } = event.target;

    this.store.setSelectedIndex(value);
  }
}
