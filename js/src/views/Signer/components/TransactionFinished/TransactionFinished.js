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

import CircularProgress from 'material-ui/CircularProgress';
import TransactionMainDetails from '../TransactionMainDetails';
import TxHashLink from '../TxHashLink';
import TransactionSecondaryDetails from '../TransactionSecondaryDetails';

import styles from './TransactionFinished.css';

import * as tUtil from '../util/transaction';
import { capitalize } from '../util/util';

export default class TransactionFinished extends Component {
  static contextTypes = {
    api: PropTypes.object.isRequired
  };

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
    data: PropTypes.string
  };

  state = {
    chain: null,
    fromBalance: null,
    toBalance: null
  };

  componentWillMount () {
    const { gas, gasPrice, value } = this.props;
    const fee = tUtil.getFee(gas, gasPrice); // BigNumber object
    const totalValue = tUtil.getTotalValue(fee, value);
    this.setState({ totalValue });

    this.context.api.ethcore.netChain()
      .then((chain) => {
        this.setState({ chain });
      })
      .catch((err) => {
        console.error('could not fetch chain', err);
      });

    const { from, to } = this.props;
    this.fetchBalance(from, 'fromBalance');
    if (to) this.fetchBalance(to, 'toBalance');
  }

  render () {
    const { chain, fromBalance, toBalance } = this.state;
    if (!chain || !fromBalance || !toBalance) {
      return (
        <div className={ `${styles.container} ${className}` }>
          <CircularProgress size={ 1 } />
        </div>
      );
    }

    const { className, date, id } = this.props;
    const { totalValue } = this.state;

    return (
      <div className={ `${styles.container} ${className || ''}` }>
        <div className={ styles.mainContainer }>
          <TransactionMainDetails
            { ...this.props }
            { ...this.state }
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
      </div>
    );
  }

  renderStatus () {
    const { status } = this.props;
    const klass = status === 'confirmed' ? styles.isConfirmed : styles.isRejected;
    return (
      <div>
        <span className={ klass }>{ capitalize(status) }</span>
        { this.renderTxHash() }
      </div>
    );
  }

  renderTxHash () {
    const { txHash, chain } = this.props;
    if (!txHash) {
      return;
    }

    return (
      <div>
        Transaction hash:
        <TxHashLink chain={ chain } txHash={ txHash } className={ styles.txHash } />
      </div>
    );
  }

  fetchBalance (address, key) {
    this.context.api.eth.getBalance(address)
      .then((balance) => {
        this.setState({ [key]: balance });
      })
      .catch((err) => {
        console.error('could not fetch balance', err);
      });
  }

}
