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

import BigNumber from 'bignumber.js';
import { observer } from 'mobx-react';
import React, { Component, PropTypes } from 'react';

import Input from '../Form/Input';
import GasPriceSelector from '../GasPriceSelector';
import Store from './store';

import styles from './gasPriceEditor.css';

@observer
export default class GasPriceEditor extends Component {
  static contextTypes = {
    api: PropTypes.object.isRequired
  };

  static propTypes = {
    children: PropTypes.node,
    onChange: PropTypes.func,
    store: PropTypes.object.isRequired
  }

  static Store = Store;

  render () {
    const { api } = this.context;
    const { children, store } = this.props;
    const { errorGas, errorPrice, errorTotal, estimated, gas, histogram, price, priceDefault, totalValue } = store;

    const eth = api.util.fromWei(totalValue).toFormat();
    const gasLabel = `gas (estimated: ${new BigNumber(estimated).toFormat()})`;
    const priceLabel = `price (current: ${new BigNumber(priceDefault).toFormat()})`;

    return (
      <div className={ styles.container }>
        <div className={ styles.graphColumn }>
          <GasPriceSelector
            histogram={ histogram }
            onChange={ this.onEditGasPrice }
            price={ price }
          />
          <div className={ styles.gasPriceDesc }>
            You can choose the gas price based on the distribution of recent included transaction gas prices. The lower the gas price is, the cheaper the transaction will be. The higher the gas price is, the faster it should get mined by the network.
          </div>
        </div>

        <div className={ styles.editColumn }>
          <div className={ styles.row }>
            <Input
              error={ errorGas }
              hint='the amount of gas to use for the transaction'
              label={ gasLabel }
              onChange={ this.onEditGas }
              value={ gas }
            />
            <Input
              error={ errorPrice }
              hint='the price of gas to use for the transaction'
              label={ priceLabel }
              onChange={ this.onEditGasPrice }
              value={ price }
            />
          </div>
          <div className={ styles.row }>
            <Input
              disabled
              error={ errorTotal }
              hint='the total amount of the transaction'
              label='total transaction amount'
              value={ `${eth} ETH` }
            />
          </div>
          <div className={ styles.row }>
            { children }
          </div>
        </div>
      </div>
    );
  }

  onEditGas = (event, gas) => {
    const { store, onChange } = this.props;

    store.setGas(gas);
    onChange && onChange('gas', gas);
  }

  onEditGasPrice = (event, price) => {
    const { store, onChange } = this.props;

    store.setPrice(price);
    onChange && onChange('gasPrice', price);
  }
}
