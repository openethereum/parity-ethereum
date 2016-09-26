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

import { Container, ContainerTitle } from '../../../ui';

import styles from '../contract.css';

export default class Events extends Component {
  static contextTypes = {
    api: PropTypes.object
  }

  static propTypes = {
    events: PropTypes.array,
    isTest: PropTypes.bool
  }

  state = {
    transactions: {}
  }

  componentDidMount () {
    this.componentWillReceiveProps(this.props);
  }

  componentWillReceiveProps (newProps) {
    this.retrieveTransactions(newProps.events);
  }

  render () {
    const { events, isTest } = this.props;
    const { transactions } = this.state;

    if (!events || !events.length) {
      return null;
    }

    const rows = events.map((event) => {
      const transaction = transactions[event.transactionHash] || {};
      const classes = `${styles.event} ${styles[event.state]}`;
      const url = `https://${isTest ? 'testnet.' : ''}etherscan.io/tx/${event.transactionHash}`;
      const keys = Object.keys(event.params).map((key, index) => {
        return <div className={ styles.key } key={ `${event.key}_key_${index}` }>{ key }</div>;
      });
      const values = Object.values(event.params).map((value, index) => {
        return (
          <div className={ styles.value } key={ `${event.key}_val_${index}` }>
            { this.renderValue(value) }
          </div>
        );
      });

      return (
        <tr className={ classes } key={ event.key }>
          <td>{ event.state === 'pending' ? 'pending' : event.blockNumber.toFormat(0) }</td>
          <td className={ styles.txhash }>
            <div>{ transaction.from }</div>
            <a href={ url } target='_blank'>{ event.transactionHash }</a>
          </td>
          <td>
            <div>{ event.type } =></div>
            { keys }
          </td>
          <td>
            <div>&nbsp;</div>
            { values }
          </td>
        </tr>
      );
    });

    return (
      <Container>
        <ContainerTitle title='events' />
        <table className={ styles.events }>
          <tbody>{ rows }</tbody>
        </table>
      </Container>
    );
  }

  renderValue (value) {
    const { api } = this.context;

    if (api.util.isInstanceOf(value, BigNumber)) {
      return value.toFormat(0);
    } else if (api.util.isArray(value)) {
      return api.util.bytesToHex(value);
    }

    return value.toString();
  }

  retrieveTransactions (events) {
    const { api } = this.context;
    const { transactions } = this.state;
    const hashes = {};

    events.forEach((event) => {
      if (!hashes[event.transactionHash] && !transactions[event.transactionHash]) {
        hashes[event.transactionHash] = true;
      }
    });

    Promise
      .all(Object.keys(hashes).map((hash) => api.eth.getTransactionByHash(hash)))
      .then((newTransactions) => {
        this.setState({
          transactions: newTransactions.reduce((store, transaction) => {
            transactions[transaction.hash] = transaction;
            return transactions;
          }, transactions)
        });
      });
  }
}
