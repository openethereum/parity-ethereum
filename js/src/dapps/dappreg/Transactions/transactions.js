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

import React, { Component } from 'react';
import { observer } from 'mobx-react';

import DappsStore from '../dappsStore';

import styles from './transactions.css';

@observer
export default class Transactions extends Component {
  dappsStore = DappsStore.get();

  render () {
    const { transactions } = this.dappsStore;
    const displayedTransactions = Object.values(transactions)
      .filter((tx) => !tx.hide)
      .sort((txA, txB) => txB.start - txA.start);

    return (
      <div className={ styles.container }>
        { displayedTransactions.map((transaction) => this.renderTransaction(transaction)) }
      </div>
    );
  }

  renderTransaction (transaction) {
    const { error, name, requestId, start, transactionHash, transactionReceipt } = transaction;

    const date = new Date(start);
    const isError = !!error;
    const isPendingNetwork = transactionHash && !transactionReceipt;
    const isConfirmed = !!transactionReceipt;

    const transactionClasses = [ styles.content ];
    const handleHideTransaction = (event) => this.handleHideTransaction(event, requestId);

    if (isError) {
      transactionClasses.push(styles.error);
    }

    if (isPendingNetwork) {
      transactionClasses.push(styles.pending);
    }

    if (isConfirmed) {
      transactionClasses.push(styles.confirmed);
    }

    return (
      <div
        className={ styles.transaction }
        key={ requestId }
      >
        <div className={ styles.header }>
          {
            name
            ? (
              <div
                className={ styles.name }
                title={ name }
              >
                { name }
              </div>
            )
            : null
          }
          <div
            className={ styles.date }
            title={ date.toISOString() }
          >
            { date.toLocaleTimeString() }
          </div>
        </div>
        <div
          className={ transactionClasses.join(' ') }
          onClick={ handleHideTransaction }
        >
          { this.renderTransactionContent(transaction) }
        </div>
      </div>
    );
  }

  renderTransactionContent (transaction) {
    const { error, transactionHash, transactionReceipt } = transaction;

    if (error) {
      return (
        <div>
          { error.text || error.message || error.toString() }
        </div>
      );
    }

    if (transactionReceipt) {
      return (
        <div>
          Transaction mined at block { transactionReceipt.blockNumber.toFormat(0) }
        </div>
      );
    }

    if (transactionHash) {
      return (
        <div>
          Transaction sent to network with hash { transactionHash }..
        </div>
      );
    }

    return (
      <div>
        Transaction waiting to be signed...
      </div>
    );
  }

  handleHideTransaction = (event, requestId) => {
    this.dappsStore.updateTransaction(requestId, { hide: true });
  }
}
