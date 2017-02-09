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

import React, { Component, PropTypes } from 'react';
import { observer } from 'mobx-react';

import { Button, GasPriceEditor } from '~/ui';

import TransactionMainDetails from '../TransactionMainDetails';
import TransactionPendingForm from '../TransactionPendingForm';

import styles from './transactionPending.css';

import * as tUtil from '../util/transaction';

@observer
export default class TransactionPending extends Component {
  static contextTypes = {
    api: PropTypes.object.isRequired
  };

  static propTypes = {
    className: PropTypes.string,
    date: PropTypes.instanceOf(Date).isRequired,
    focus: PropTypes.bool,
    gasLimit: PropTypes.object,
    id: PropTypes.object.isRequired,
    isSending: PropTypes.bool.isRequired,
    isTest: PropTypes.bool.isRequired,
    nonce: PropTypes.number,
    onConfirm: PropTypes.func.isRequired,
    onReject: PropTypes.func.isRequired,
    store: PropTypes.object.isRequired,
    transaction: PropTypes.shape({
      condition: PropTypes.object,
      data: PropTypes.string,
      from: PropTypes.string.isRequired,
      gas: PropTypes.object.isRequired,
      gasPrice: PropTypes.object.isRequired,
      to: PropTypes.string,
      value: PropTypes.object.isRequired
    }).isRequired
  };

  static defaultProps = {
    focus: false
  };

  gasStore = new GasPriceEditor.Store(this.context.api, {
    condition: this.props.transaction.condition,
    gas: this.props.transaction.gas.toFixed(),
    gasLimit: this.props.gasLimit,
    gasPrice: this.props.transaction.gasPrice.toFixed()
  });

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
      ? this.renderTxEditor()
      : this.renderTransaction();
  }

  renderTransaction () {
    const { className, focus, id, isSending, isTest, store, transaction } = this.props;
    const { totalValue } = this.state;
    const { balances, externalLink } = store;
    const { from, value } = transaction;

    const fromBalance = balances[from];

    return (
      <div className={ `${styles.container} ${className}` }>
        <TransactionMainDetails
          className={ styles.transactionDetails }
          externalLink={ externalLink }
          from={ from }
          fromBalance={ fromBalance }
          gasStore={ this.gasStore }
          id={ id }
          isTest={ isTest }
          totalValue={ totalValue }
          transaction={ transaction }
          value={ value }
        />
        <TransactionPendingForm
          address={ from }
          focus={ focus }
          isSending={ isSending }
          onConfirm={ this.onConfirm }
          onReject={ this.onReject }
        />
      </div>
    );
  }

  renderTxEditor () {
    const { className } = this.props;

    return (
      <div className={ `${styles.container} ${className}` }>
        <GasPriceEditor store={ this.gasStore }>
          <Button
            label='view transaction'
            onClick={ this.toggleGasEditor }
          />
        </GasPriceEditor>
      </div>
    );
  }

  onConfirm = (data) => {
    const { id, transaction } = this.props;
    const { password, wallet } = data;
    const { condition, gas, gasPrice } = this.gasStore.overrideTransaction(transaction);

    const options = {
      gas,
      gasPrice,
      id,
      password,
      wallet
    };

    if (condition && (condition.block || condition.time)) {
      options.condition = condition;
    }

    this.props.onConfirm(options);
  }

  onReject = () => {
    this.props.onReject(this.props.id);
  }

  toggleGasEditor = () => {
    this.gasStore.setEditing(false);
  }
}
