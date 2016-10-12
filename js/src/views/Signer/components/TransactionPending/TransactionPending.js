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

import TransactionMainDetails from '../TransactionMainDetails';
import TransactionPendingForm from '../TransactionPendingForm';
import TransactionSecondaryDetails from '../TransactionSecondaryDetails';

import styles from './TransactionPending.css';

import * as tUtil from '../util/transaction';

export default class TransactionPending extends Component {

  static propTypes = {
    id: PropTypes.string.isRequired,
    chain: PropTypes.string.isRequired,
    from: PropTypes.string.isRequired,
    fromBalance: PropTypes.object, // eth BigNumber, not required since it mght take time to fetch
    value: PropTypes.string.isRequired, // wei hex
    gasPrice: PropTypes.string.isRequired, // wei hex
    gas: PropTypes.string.isRequired, // hex
    date: PropTypes.instanceOf(Date).isRequired,
    to: PropTypes.string, // undefined if it's a contract
    toBalance: PropTypes.object, // eth BigNumber - undefined if it's a contract or until it's fetched
    data: PropTypes.string, // hex
    nonce: PropTypes.number,
    onConfirm: PropTypes.func.isRequired,
    onReject: PropTypes.func.isRequired,
    isSending: PropTypes.bool.isRequired,
    className: PropTypes.string
  };

  static defaultProps = {
    isSending: false
  };

  state = {
    isDataExpanded: false
  };

  componentWillMount () {
    const { gas, gasPrice, value } = this.props;
    const fee = tUtil.getFee(gas, gasPrice); // BigNumber object
    const totalValue = tUtil.getTotalValue(fee, value);
    const gasPriceEthmDisplay = tUtil.getEthmFromWeiDisplay(gasPrice);
    const gasToDisplay = tUtil.getGasDisplay(gas);
    this.setState({ gasPriceEthmDisplay, totalValue, gasToDisplay });
  }

  render () {
    const { totalValue } = this.state;
    const className = this.props.className || '';

    const { gasPriceEthmDisplay, gasToDisplay } = this.state;
    const { id, date, data, from } = this.props;

    return (
      <div className={ `${styles.container} ${className}` }>
        <div className={ styles.mainContainer }>
          <TransactionMainDetails
            { ...this.props }
            className={ styles.transactionDetails }
            totalValue={ totalValue }>
            <TransactionSecondaryDetails
              id={ id }
              date={ date }
              data={ data }
              gasPriceEthmDisplay={ gasPriceEthmDisplay }
              gasToDisplay={ gasToDisplay }
            />
          </TransactionMainDetails>
          <TransactionPendingForm
            address={ from }
            isSending={ this.props.isSending }
            onConfirm={ this.onConfirm }
            onReject={ this.onReject }
          />
        </div>
      </div>
    );
  }

  onConfirm = password => {
    const { id, gasPrice } = this.props;
    this.props.onConfirm({ id, password, gasPrice });
  }

  onReject = () => {
    this.props.onReject(this.props.id);
  }

}
