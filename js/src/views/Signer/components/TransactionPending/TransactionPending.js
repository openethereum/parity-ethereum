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
import TransactionPendingForm from '../TransactionPendingForm';
import TransactionSecondaryDetails from '../TransactionSecondaryDetails';

import styles from './TransactionPending.css';

import * as tUtil from '../util/transaction';

export default class TransactionPending extends Component {
  static contextTypes = {
    api: PropTypes.object.isRequired
  };

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
    className: PropTypes.string
  };

  static defaultProps = {
    isSending: false
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
    const gasPriceEthmDisplay = tUtil.getEthmFromWeiDisplay(gasPrice);
    const gasToDisplay = tUtil.getGasDisplay(gas);
    this.setState({ gasPriceEthmDisplay, totalValue, gasToDisplay });

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

    const { totalValue, gasPriceEthmDisplay, gasToDisplay } = this.state;
    const { className, id, date, data, from } = this.props;

    return (
      <div className={ `${styles.container} ${className || ''}` }>
        <div className={ styles.mainContainer }>
          <TransactionMainDetails
            { ...this.props }
            { ...this.state }
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
      </div>
    );
  }

  onConfirm = password => {
    const { id, gasPrice } = this.props;
    this.props.onConfirm({ id, password, gasPrice });
  }

  onReject = () => {
    this.props.onReject(this.props.id);
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
