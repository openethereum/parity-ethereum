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

import { Button, GasPriceEditor } from '~/ui';

import TransactionMainDetails from '../TransactionMainDetails';
import TransactionPendingForm from '../TransactionPendingForm';

import styles from './TransactionPending.css';

import * as tUtil from '../util/transaction';

@observer
export default class TransactionPending extends Component {
  static contextTypes = {
    api: PropTypes.object.isRequired
  };

  static propTypes = {
    className: PropTypes.string,
    date: PropTypes.instanceOf(Date).isRequired,
    gasLimit: PropTypes.object,
    id: PropTypes.object.isRequired,
    isSending: PropTypes.bool.isRequired,
    isTest: PropTypes.bool.isRequired,
    nonce: PropTypes.number,
    onConfirm: PropTypes.func.isRequired,
    onReject: PropTypes.func.isRequired,
    store: PropTypes.object.isRequired,
    transaction: PropTypes.shape({
      from: PropTypes.string.isRequired,
      value: PropTypes.object.isRequired, // wei hex
      gasPrice: PropTypes.object.isRequired, // wei hex
      gas: PropTypes.object.isRequired, // hex
      data: PropTypes.string, // hex
      to: PropTypes.string // undefined if it's a contract
    }).isRequired
  };

  static defaultProps = {
    isSending: false
  };

  gasStore = new GasPriceEditor.Store(this.context.api, this.props.gasLimit);

  componentWillMount () {
    const { store, transaction } = this.props;
    const { from, gas, gasPrice, to, value } = transaction;

    const fee = tUtil.getFee(gas, gasPrice); // BigNumber object
    const gasPriceEthmDisplay = tUtil.getEthmFromWeiDisplay(gasPrice);
    const gasToDisplay = tUtil.getGasDisplay(gas);
    const totalValue = tUtil.getTotalValue(fee, value);

    this.setState({ gasPriceEthmDisplay, totalValue, gasToDisplay });
    this.gasStore.setEthValue(value);

    store.fetchBalances([from, to]);
  }

  render () {
    return this.gasStore.isEditing
      ? this.renderGasEditor()
      : this.renderTransaction();
  }

  renderTransaction () {
    const { className, id, isTest, store, transaction } = this.props;
    const { totalValue } = this.state;
    const { from, value } = transaction;

    const fromBalance = store.balances[from];

    return (
      <div className={ `${styles.container} ${className || ''}` }>
        <TransactionMainDetails
          className={ styles.transactionDetails }
          from={ from }
          fromBalance={ fromBalance }
          gasStore={ this.gasStore }
          id={ id }
          isTest={ isTest }
          totalValue={ totalValue }
          transaction={ transaction }
          value={ value } />
        <TransactionPendingForm
          address={ from }
          isSending={ this.props.isSending }
          onConfirm={ this.onConfirm }
          onReject={ this.onReject } />
      </div>
    );
  }

  renderGasEditor () {
    // const { gasLimit } = this.props;
    //
    // this.gasStore.setGasLimit(gasLimit);

    return (
      <GasPriceEditor
        store={ this.gasStore }>
        <Button
          label='view transaction'
          onClick={ this.toggleGasEditor } />
      </GasPriceEditor>
    );
  }

  onConfirm = data => {
    const { id, transaction } = this.props;
    const { gasPrice } = transaction;
    const { password, wallet } = data;

    this.props.onConfirm({ id, password, wallet, gasPrice });
  }

  onReject = () => {
    this.props.onReject(this.props.id);
  }

  toggleGasEditor = () => {
    this.gasStore.setEditing(false);
  }
}
