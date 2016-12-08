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

import BigNumber from 'bignumber.js';
import React, { Component, PropTypes } from 'react';

import Input from '../Form/Input';
import GasPriceSelector from './GasPriceSelector';
import Store from './store';

import styles from './gasPriceEditor.css';

export default class GasPriceEditor extends Component {
  static contextTypes = {
    api: PropTypes.object.isRequired
  };

  static propTypes = {
    store: PropTypes.object.isRequired
  }

  static Store = Store;

  render () {
    const { api } = this.context;
    const { store } = this.props;

    const eth = api.util.fromWei(store.ethTotal).toFormat(6);
    const gasLabel = `gas amount (estimated: ${new BigNumber(store.estimated).toFormat()})`;
    const priceLabel = `gas price (current: ${new BigNumber(store.priceDefault).toFormat()})`;

    return (
      <div className={ styles.columns }>
        <div className={ styles.graphColumn }>
          <GasPriceSelector
            gasPriceHistogram={ store.histogram }
            gasPrice={ store.price }
            onChange={ this.onEditGasPrice } />
          <div>
            <p className={ styles.gasPriceDesc }>
              You can choose the gas price based on the
              distribution of recent included transaction gas prices.
              The lower the gas price is, the cheaper the transaction will
              be. The higher the gas price is, the faster it should
              get mined by the network.
            </p>
          </div>
        </div>

        <div className={ styles.editColumn }>
          <div className={ styles.row }>
            <Input
              label={ gasLabel }
              hint='the amount of gas to use for the transaction'
              error={ store.errorGas }
              value={ store.gas }
              onChange={ this.onEditGas } />

            <Input
              label={ priceLabel }
              hint='the price of gas to use for the transaction'
              error={ store.errorPrice }
              value={ store.price }
              onChange={ this.onEditGasPrice } />
          </div>

          <div className={ styles.row }>
            <Input
              disabled
              label='total transaction amount'
              hint='the total amount of the transaction'
              error={ totalError }
              value={ `${eth} ETH` } />
          </div>
        </div>
      </div>
    );
  }

  onEditGas = (event) => {
    this.props.onChange('gas', event.target.value);
  }

  onEditGasPrice = (event, value) => {
    this.props.onChange('gasPrice', value);
  }
}
