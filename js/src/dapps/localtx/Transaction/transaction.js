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
import React, { Component, PropTypes } from 'react';
import classnames from 'classnames';

import { api } from '../parity';

import styles from './transaction.css';

import IdentityIcon from '../IdentityIcon';

class BaseTransaction extends Component {
  shortHash (hash) {
    return `${hash.substr(0, 5)}..${hash.substr(hash.length - 3)}`;
  }

  renderHash (hash) {
    return (
      <code title={ hash } className={ styles.txhash }>
        { hash }
      </code>
    );
  }

  renderFrom (transaction) {
    if (!transaction) {
      return '-';
    }

    return (
      <div title={ transaction.from } className={ styles.from }>
        <IdentityIcon
          address={ transaction.from }
        />
      </div>
    );
  }

  renderGasPrice (transaction) {
    if (!transaction) {
      return '-';
    }

    return (
      <span title={ `${transaction.gasPrice.toFormat(0)} wei` }>
        { api.util.fromWei(transaction.gasPrice, 'shannon').toFormat(2) }&nbsp;shannon
      </span>
    );
  }

  renderGas (transaction) {
    if (!transaction) {
      return '-';
    }

    return (
      <span title={ `${transaction.gas.toFormat(0)} Gas` }>
        { transaction.gas.div(10 ** 6).toFormat(3) }&nbsp;MGas
      </span>
    );
  }

  renderPropagation (stats) {
    const noOfPeers = Object.keys(stats.propagatedTo).length;
    const noOfPropagations = Object.values(stats.propagatedTo).reduce((sum, val) => sum + val, 0);

    return (
      <span className={ styles.nowrap }>
        { noOfPropagations } ({ noOfPeers } peers)
      </span>
    );
  }
}

export class Transaction extends BaseTransaction {
  static propTypes = {
    idx: PropTypes.number.isRequired,
    transaction: PropTypes.object.isRequired,
    blockNumber: PropTypes.object.isRequired,
    isLocal: PropTypes.bool,
    stats: PropTypes.object
  };

  static defaultProps = {
    isLocal: false,
    stats: {
      firstSeen: 0,
      propagatedTo: {}
    }
  };

  static renderHeader () {
    return (
      <tr className={ styles.header }>
        <th />
        <th>
          Transaction
        </th>
        <th>
          From
        </th>
        <th>
          Gas Price
        </th>
        <th>
          Gas
        </th>
        <th>
          First propagation
        </th>
        <th>
          # Propagated
        </th>
        <th />
      </tr>
    );
  }

  render () {
    const { isLocal, stats, transaction, idx } = this.props;
    const blockNo = new BigNumber(stats.firstSeen);

    const clazz = classnames(styles.transaction, {
      [styles.local]: isLocal
    });

    return (
      <tr className={ clazz }>
        <td>
          { idx }.
        </td>
        <td>
          { this.renderHash(transaction.hash) }
        </td>
        <td>
          { this.renderFrom(transaction) }
        </td>
        <td>
          { this.renderGasPrice(transaction) }
        </td>
        <td>
          { this.renderGas(transaction) }
        </td>
        <td title={ blockNo.toFormat(0) }>
          { this.renderTime(stats.firstSeen) }
        </td>
        <td>
          { this.renderPropagation(stats) }
        </td>
      </tr>
    );
  }

  renderTime (firstSeen) {
    const { blockNumber } = this.props;

    if (!firstSeen) {
      return 'never';
    }

    const timeInMinutes = blockNumber.sub(firstSeen).mul(14).div(60).toFormat(1);

    return `${timeInMinutes} minutes ago`;
  }
}

export class LocalTransaction extends BaseTransaction {
  static propTypes = {
    hash: PropTypes.string.isRequired,
    status: PropTypes.string.isRequired,
    transaction: PropTypes.object,
    isLocal: PropTypes.bool,
    stats: PropTypes.object,
    details: PropTypes.object
  };

  static defaultProps = {
    stats: {
      propagatedTo: {}
    }
  };

  static renderHeader () {
    return (
      <tr className={ styles.header }>
        <th />
        <th>
          Transaction
        </th>
        <th>
          From
        </th>
        <th>
          Gas Price
        </th>
        <th>
          Gas
        </th>
        <th>
          Status
        </th>
      </tr>
    );
  }

  state = {
    isSending: false,
    isResubmitting: false,
    gasPrice: null,
    gas: null
  };

  toggleResubmit = () => {
    const { transaction } = this.props;
    const { isResubmitting } = this.state;

    const nextState = {
      isResubmitting: !isResubmitting
    };

    if (!isResubmitting) {
      nextState.gasPrice = api.util.fromWei(transaction.gasPrice, 'shannon').toNumber();
      nextState.gas = transaction.gas.div(1000000).toNumber();
    }

    this.setState(nextState);
  };

  setGasPrice = el => {
    this.setState({
      gasPrice: el.target.value
    });
  };

  setGas = el => {
    this.setState({
      gas: el.target.value
    });
  };

  sendTransaction = () => {
    const { transaction, status } = this.props;
    const { gasPrice, gas } = this.state;

    const newTransaction = {
      from: transaction.from,
      value: transaction.value,
      data: transaction.input,
      gasPrice: api.util.toWei(gasPrice, 'shannon'),
      gas: new BigNumber(gas).mul(1000000)
    };

    this.setState({
      isResubmitting: false,
      isSending: true
    });

    const closeSending = () => {
      this.setState({
        isSending: false,
        gasPrice: null,
        gas: null
      });
    };

    if (transaction.to) {
      newTransaction.to = transaction.to;
    }

    if (!['mined', 'replaced'].includes(status)) {
      newTransaction.nonce = transaction.nonce;
    }

    api.eth.sendTransaction(newTransaction)
      .then(closeSending)
      .catch(closeSending);
  };

  render () {
    if (this.state.isResubmitting) {
      return this.renderResubmit();
    }

    const { stats, transaction, hash, status } = this.props;
    const { isSending } = this.state;

    const resubmit = isSending ? (
      'sending...'
    ) : (
      <button onClick={ this.toggleResubmit }>
        resubmit
      </button>
    );

    return (
      <tr className={ styles.transaction }>
        <td>
          { !transaction ? null : resubmit }
        </td>
        <td>
          { this.renderHash(hash) }
        </td>
        <td>
          { this.renderFrom(transaction) }
        </td>
        <td>
          { this.renderGasPrice(transaction) }
        </td>
        <td>
          { this.renderGas(transaction) }
        </td>
        <td>
          { this.renderStatus() }
          <br />
          { status === 'pending' ? this.renderPropagation(stats) : null }
        </td>
      </tr>
    );
  }

  renderStatus () {
    const { details } = this.props;

    let state = {
      'pending': () => 'In queue: Pending',
      'future': () => 'In queue: Future',
      'mined': () => 'Mined',
      'dropped': () => 'Dropped because of queue limit',
      'invalid': () => 'Transaction is invalid',
      'rejected': () => `Rejected: ${details.error}`,
      'replaced': () => `Replaced by ${this.shortHash(details.hash)}`
    }[this.props.status];

    return state ? state() : 'unknown';
  }

  // TODO [ToDr] Gas Price / Gas selection is not needed
  // when signer supports gasPrice/gas tunning.
  renderResubmit () {
    const { transaction } = this.props;
    const { gasPrice, gas } = this.state;

    return (
      <tr className={ styles.transaction }>
        <td>
          <button onClick={ this.toggleResubmit }>
            cancel
          </button>
        </td>
        <td>
          { this.renderHash(transaction.hash) }
        </td>
        <td>
          { this.renderFrom(transaction) }
        </td>
        <td className={ styles.edit }>
          <input
            type='number'
            value={ gasPrice }
            onChange={ this.setGasPrice }
          />
          <span>shannon</span>
        </td>
        <td className={ styles.edit }>
          <input
            type='number'
            value={ gas }
            onChange={ this.setGas }
          />
          <span>MGas</span>
        </td>
        <td colSpan='2'>
          <button onClick={ this.sendTransaction }>
            Send
          </button>
        </td>
      </tr>
    );
  }
}
