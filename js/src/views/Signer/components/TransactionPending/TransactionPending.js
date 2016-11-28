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
import { observer } from 'mobx-react';

import TransactionMainDetails from '../TransactionMainDetails';
import TransactionPendingForm from '../TransactionPendingForm';
import TransactionSecondaryDetails from '../TransactionSecondaryDetails';

import styles from './TransactionPending.css';

import * as tUtil from '../util/transaction';

@observer
export default class TransactionPending extends Component {
  static propTypes = {
    id: PropTypes.object.isRequired,
    from: PropTypes.string.isRequired,
    value: PropTypes.object.isRequired, // wei hex
    gasPrice: PropTypes.object.isRequired, // wei hex
    gas: PropTypes.object.isRequired, // hex
    date: PropTypes.instanceOf(Date).isRequired,
    to: PropTypes.string, // undefined if it's a contract
    data: PropTypes.string, // hex
    nonce: PropTypes.number,
    onConfirm: PropTypes.func.isRequired,
    onReject: PropTypes.func.isRequired,
    isSending: PropTypes.bool.isRequired,
    className: PropTypes.string,
    isTest: PropTypes.bool.isRequired,
    store: PropTypes.object.isRequired
  };

  static defaultProps = {
    isSending: false
  };

  componentWillMount () {
    const { gas, gasPrice, value, from, to, store } = this.props;

    const fee = tUtil.getFee(gas, gasPrice); // BigNumber object
    const totalValue = tUtil.getTotalValue(fee, value);
    const gasPriceEthmDisplay = tUtil.getEthmFromWeiDisplay(gasPrice);
    const gasToDisplay = tUtil.getGasDisplay(gas);

    this.setState({ gasPriceEthmDisplay, totalValue, gasToDisplay });
    store.fetchBalances([from, to]);
  }

  render () {
    const { className, id, date, data, from, to, store } = this.props;
    const { totalValue, gasPriceEthmDisplay, gasToDisplay } = this.state;

    const fromBalance = store.balances[from];
    const toBalance = store.balances[to];

    return (
      <div className={ `${styles.container} ${className || ''}` }>
        <TransactionMainDetails
          { ...this.props }
          { ...this.state }
          fromBalance={ fromBalance }
          toBalance={ toBalance }
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
    );
  }

  onConfirm = data => {
    const { id, gasPrice } = this.props;
    const { password, wallet } = data;

    this.props.onConfirm({ id, password, wallet, gasPrice });
  }

  onReject = () => {
    this.props.onReject(this.props.id);
  }
}
