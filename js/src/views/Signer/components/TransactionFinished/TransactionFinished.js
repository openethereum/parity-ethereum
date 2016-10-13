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
import TxHashLink from '../TxHashLink';
import TransactionSecondaryDetails from '../TransactionSecondaryDetails';

import styles from './TransactionFinished.css';

import * as tUtil from '../util/transaction';
import { capitalize } from '../util/util';

export default class TransactionFinished extends Component {
  static propTypes = {
    id: PropTypes.object.isRequired,
    from: PropTypes.string.isRequired,
    fromBalance: PropTypes.object, // eth BigNumber, not required since it might take time to fetch
    value: PropTypes.object.isRequired, // wei hex
    chain: PropTypes.string.isRequired,
    gasPrice: PropTypes.object.isRequired, // wei hex
    gas: PropTypes.object.isRequired, // hex
    status: PropTypes.string.isRequired, // rejected, confirmed
    date: PropTypes.instanceOf(Date).isRequired,
    to: PropTypes.string, // undefined if it's a contract
    toBalance: PropTypes.object, // eth BigNumber - undefined if it's a contract or until it's fetched
    txHash: PropTypes.string, // undefined if transacation is rejected
    className: PropTypes.string,
    data: PropTypes.string
  };

  static defaultProps = {
    value: '0x0' // todo [adgo] - remove after resolving https://github.com/ethcore/parity/issues/1458
  };

  componentWillMount () {
    const { gas, gasPrice, value } = this.props;
    const fee = tUtil.getFee(gas, gasPrice); // BigNumber object
    const totalValue = tUtil.getTotalValue(fee, value);
    this.setState({ totalValue });
  }

  render () {
    const { className, date, id } = this.props;
    const { totalValue } = this.state;

    return (
      <div className={ `${styles.container} ${className || ''}` }>
        <div className={ styles.mainContainer }>
          <TransactionMainDetails
            { ...this.props }
            totalValue={ totalValue }
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
        Transaction hash: <br />
        <TxHashLink chain={ chain } txHash={ txHash } className={ styles.txHash } />
      </div>
    );
  }

}
