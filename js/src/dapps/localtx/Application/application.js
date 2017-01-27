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

import BigNumber from 'bignumber.js';
import React, { Component } from 'react';

import { api } from '../parity';

import styles from './application.css';

import { Transaction, LocalTransaction } from '../Transaction';

export default class Application extends Component {
  state = {
    loading: true,
    transactions: [],
    localTransactions: {},
    blockNumber: 0
  }

  componentDidMount () {
    const poll = () => {
      this._timeout = window.setTimeout(() => {
        this.fetchTransactionData().then(poll).catch(poll);
      }, 1000);
    };

    poll();
  }

  componentWillUnmount () {
    clearTimeout(this._timeout);
  }

  fetchTransactionData () {
    return Promise.all([
      api.parity.pendingTransactions(),
      api.parity.pendingTransactionsStats(),
      api.parity.localTransactions(),
      api.eth.blockNumber()
    ]).then(([pending, stats, local, blockNumber]) => {
      // Combine results together
      const transactions = pending.map(tx => {
        return {
          transaction: tx,
          stats: stats[tx.hash],
          isLocal: !!local[tx.hash]
        };
      });

      // Add transaction data to locals
      transactions
        .filter(tx => tx.isLocal)
        .map(data => {
          const tx = data.transaction;

          local[tx.hash].transaction = tx;
          local[tx.hash].stats = data.stats;
        });

      // Convert local transactions to array
      const localTransactions = Object.keys(local).map(hash => {
        const data = local[hash];

        data.txHash = hash;
        return data;
      });

      // Sort local transactions by nonce (move future to the end)
      localTransactions.sort((a, b) => {
        a = a.transaction || {};
        b = b.transaction || {};

        if (a.from && b.from && a.from !== b.from) {
          return a.from < b.from;
        }

        if (!a.nonce || !b.nonce) {
          return !a.nonce ? 1 : -1;
        }

        return new BigNumber(a.nonce).comparedTo(new BigNumber(b.nonce));
      });

      this.setState({
        loading: false,
        transactions,
        localTransactions,
        blockNumber
      });
    });
  }

  render () {
    const { loading } = this.state;

    if (loading) {
      return (
        <div className={ styles.container }>Loading...</div>
      );
    }

    return (
      <div className={ styles.container }>
        <h1>Your local transactions</h1>
        { this.renderLocals() }
        <h1>Transactions in the queue</h1>
        { this.renderQueueSummary() }
        { this.renderQueue() }
      </div>
    );
  }

  renderQueueSummary () {
    const { transactions } = this.state;

    if (!transactions.length) {
      return null;
    }

    const count = transactions.length;
    const locals = transactions.filter(tx => tx.isLocal).length;
    const fee = transactions
      .map(tx => tx.transaction)
      .map(tx => tx.gasPrice.mul(tx.gas))
      .reduce((sum, fee) => sum.add(fee), new BigNumber(0));

    return (
      <h3>
        Count: <strong>{ locals ? `${count} (${locals})` : count }</strong>
        &nbsp;
        Total Fee: <strong>{ api.util.fromWei(fee).toFixed(3) } ETH</strong>
      </h3>
    );
  }

  renderQueue () {
    const { blockNumber, transactions } = this.state;

    if (!transactions.length) {
      return (
        <h3>The queue seems is empty.</h3>
      );
    }

    return (
      <table cellSpacing='0'>
        <thead>
          { Transaction.renderHeader() }
        </thead>
        <tbody>
          {
            transactions.map((tx, idx) => (
              <Transaction
                key={ tx.transaction.hash }
                idx={ idx + 1 }
                isLocal={ tx.isLocal }
                transaction={ tx.transaction }
                stats={ tx.stats }
                blockNumber={ blockNumber }
              />
            ))
          }
        </tbody>
      </table>
    );
  }

  renderLocals () {
    const { localTransactions } = this.state;

    if (!localTransactions.length) {
      return (
        <h3>You haven't sent any transactions yet.</h3>
      );
    }

    return (
      <table cellSpacing='0'>
        <thead>
          { LocalTransaction.renderHeader() }
        </thead>
        <tbody>
          {
            localTransactions.map(tx => (
              <LocalTransaction
                key={ tx.txHash }
                hash={ tx.txHash }
                transaction={ tx.transaction }
                status={ tx.status }
                stats={ tx.stats }
                details={ tx }
              />
            ))
          }
        </tbody>
      </table>
    );
  }
}
