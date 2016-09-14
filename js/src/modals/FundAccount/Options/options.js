// Copyright 2015, 2016 Ethcore (UK) Ltd.
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

import React, { Component, PropTypes } from 'react';
import { MenuItem } from 'material-ui';

import { Form, Input, Select } from '../../../ui';

import styles from './options.css';

export default class Options extends Component {
  static propTypes = {
    coinSymbol: PropTypes.string.isRequired,
    coins: PropTypes.array.isRequired
  };

  render () {
    const { coinSymbol, coins } = this.props;
    const label = `(optional) ${coinSymbol} return address`;

    if (!coins.length) {
      return (
        <div className={ styles.nocoins }>
          There are currently no coins available to fund with.
        </div>
      );
    }

    const items = coins.map(this.renderCoinSelectItem);

    return (
      <Form>
        <Select
          className={ styles.coinselector }
          label='fund account from'
          hint='the type of crypto conversion to do'
          value={ coinSymbol }
          onChange={ this.onSelectCoin }>
          { items }
        </Select>
        <Input
          label={ label }
          hint='the return address for send failures' />
      </Form>
    );
  }

  renderCoinSelectItem = (coin) => {
    const { image, name, symbol } = coin;

    const item = (
      <div className={ styles.coinselect }>
        <img className={ styles.coinimage } src={ image } />
        <div className={ styles.coindetails }>
          <div className={ styles.coinsymbol }>
            { symbol }
          </div>
          <div className={ styles.coinname }>
            { name }
          </div>
        </div>
      </div>
    );

    return (
      <MenuItem
        key={ symbol }
        value={ symbol }
        label={ item }>
        { item }
      </MenuItem>
    );
  }

  onSelectCoin = (event, idx, value) => {
    console.log(idx, value);
  }
}
