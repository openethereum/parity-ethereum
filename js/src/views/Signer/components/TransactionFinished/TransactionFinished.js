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

import { TxHash } from '../../../../ui';

import TransactionMainDetails from '../TransactionMainDetails';
import TxHashLink from '../TxHashLink';
import TransactionSecondaryDetails from '../TransactionSecondaryDetails';

import styles from './TransactionFinished.css';

import * as tUtil from '../util/transaction';
import { capitalize } from '../util/util';

@observer
export default class TransactionFinished extends Component {
  static propTypes = {
    id: PropTypes.object.isRequired,
    from: PropTypes.string.isRequired,
    value: PropTypes.object.isRequired, // wei hex
    gasPrice: PropTypes.object.isRequired, // wei hex
    gas: PropTypes.object.isRequired, // hex
    status: PropTypes.string.isRequired, // rejected, confirmed
    date: PropTypes.instanceOf(Date).isRequired,
    to: PropTypes.string, // undefined if it's a contract
    txHash: PropTypes.string, // undefined if transacation is rejected
    className: PropTypes.string,
    data: PropTypes.string,
    isTest: PropTypes.bool.isRequired,
    store: PropTypes.object.isRequired
  };

  componentWillMount () {
    const { from, to, gas, gasPrice, value, store } = this.props;
    const fee = tUtil.getFee(gas, gasPrice); // BigNumber object
    const totalValue = tUtil.getTotalValue(fee, value);

    this.setState({ totalValue });
    store.fetchBalances([from, to]);
  }

  render () {
    const { className, date, id, from, to, store } = this.props;

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
        >
          <TransactionSecondaryDetails
            id={ id }
            date={ date }
          />
        </TransactionMainDetails>
        <div className={ styles.statusContainer }>
          { this.renderStatus() }
        </div>
      </div>
    );
  }

  renderStatus () {
    const { status, txHash } = this.props;

    if (status !== 'confirmed') {
      return (
        <div>
          <span className={ styles.isRejected }>{ capitalize(status) }</span>
        </div>
      );
    }

    return (
      <TxHash
        summary
        hash={ txHash } />
    );
  }

  renderTxHash () {
    const { txHash, isTest } = this.props;

    if (!txHash) {
      return;
    }

    return (
      <div>
        Transaction hash:
        <TxHashLink
          isTest={ isTest }
          txHash={ txHash }
          className={ styles.txHash } />
      </div>
    );
  }
}
