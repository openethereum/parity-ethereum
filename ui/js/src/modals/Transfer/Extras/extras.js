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

import Form, { Input } from '../../../ui/Form';

import styles from '../transfer.css';

export default class Extras extends Component {
  static propTypes = {
    isEth: PropTypes.bool,
    data: PropTypes.string,
    dataError: PropTypes.string,
    gas: PropTypes.string,
    gasEst: PropTypes.string,
    gasError: PropTypes.string,
    gasPrice: PropTypes.string,
    gasPriceDefault: PropTypes.string,
    gasPriceError: PropTypes.string,
    total: PropTypes.string,
    totalError: PropTypes.string,
    onChange: PropTypes.func.isRequired
  }

  render () {
    const { gas, gasError, gasEst, gasPrice, gasPriceDefault, gasPriceError, total, totalError } = this.props;
    const gasLabel = `gas amount (estimated: ${gasEst})`;
    const priceLabel = `gas price (current: ${gasPriceDefault})`;

    return (
      <Form>
        { this.renderData() }
        <div className={ styles.columns }>
          <div>
            <Input
              label={ gasLabel }
              hint='the amount of gas to use for the transaction'
              error={ gasError }
              value={ gas }
              onChange={ this.onEditGas } />
          </div>
          <div>
            <Input
              label={ priceLabel }
              hint='the price of gas to use for the transaction'
              error={ gasPriceError }
              value={ gasPrice }
              onChange={ this.onEditGasPrice } />
          </div>
        </div>
        <div className={ styles.columns }>
          <div>
            <Input
              disabled
              label='total transaction amount'
              hint='the total amount of the transaction'
              error={ totalError }
              value={ `${total} ETH` } />
          </div>
        </div>
      </Form>
    );
  }

  renderData () {
    const { isEth, data, dataError } = this.props;

    if (!isEth) {
      return null;
    }

    return (
      <div>
        <Input
          hint='the data to pass through with the transaction'
          label='transaction data'
          value={ data }
          error={ dataError }
          onChange={ this.onEditData } />
      </div>
    );
  }

  onEditGas = (event) => {
    this.props.onChange('gas', event.target.value);
  }

  onEditGasPrice = (event) => {
    this.props.onChange('gasPrice', event.target.value);
  }

  onEditData = (event) => {
    this.props.onChange('data', event.target.value);
  }
}
