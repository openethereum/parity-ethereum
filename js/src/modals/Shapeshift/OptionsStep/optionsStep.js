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
import { Checkbox, MenuItem } from 'material-ui';

import { Form, Input, Select } from '../../../ui';

import Price from '../Price';

import styles from './optionsStep.css';

export default class OptionsStep extends Component {
  static propTypes = {
    refundAddress: PropTypes.string.isRequired,
    coinSymbol: PropTypes.string.isRequired,
    coins: PropTypes.array.isRequired,
    price: PropTypes.object,
    hasAccepted: PropTypes.bool.isRequired,
    onChangeSymbol: PropTypes.func.isRequired,
    onChangeRefund: PropTypes.func.isRequired,
    onToggleAccept: PropTypes.func.isRequired
  };

  render () {
    const { coinSymbol, coins, refundAddress, hasAccepted, onToggleAccept } = this.props;
    const label = `(optional) ${coinSymbol} return address`;

    if (!coins.length) {
      return (
        <div className={ styles.empty }>
          There are currently no exchange pairs/coins available to fund with.
        </div>
      );
    }

    const items = coins.map(this.renderCoinSelectItem);

    return (
      <div className={ styles.body }>
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
            hint='the return address for send failures'
            value={ refundAddress }
            onSubmit={ this.onChangeRefund } />
          <Checkbox
            className={ styles.accept }
            label='I understand that ShapeShift.io is a 3rd-party service and by using the service any transfer of information and/or funds is completely out of the control of Parity'
            checked={ hasAccepted }
            onCheck={ onToggleAccept } />
        </Form>
        <Price { ...this.props } />
      </div>
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
    this.props.onChangeSymbol(event, value);
  }

  onChangeAddress = (event, value) => {
    this.props.onChangeRefund(value);
  }
}
