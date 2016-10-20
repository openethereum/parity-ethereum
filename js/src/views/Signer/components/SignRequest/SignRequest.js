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

import Account from '../Account';
import TransactionPendingForm from '../TransactionPendingForm';
import TxHashLink from '../TxHashLink';

import styles from './SignRequest.css';

export default class SignRequest extends Component {

  // TODO [todr] re-use proptypes?
  static propTypes = {
    id: PropTypes.string.isRequired,
    address: PropTypes.string.isRequired,
    hash: PropTypes.string.isRequired,
    isFinished: PropTypes.bool.isRequired,
    isSending: PropTypes.bool,
    onConfirm: PropTypes.func,
    onReject: PropTypes.func,
    status: PropTypes.string,
    className: PropTypes.string
  };

  state = {
    chain: null,
    balance: null
  }

  componentWillMount () {
    this.context.api.ethcore.netChain()
      .then((chain) => {
        this.setState({ chain });
      })
      .catch((err) => {
        console.error('could not fetch chain', err);
      });

    this.context.api.eth.getBalance(this.props.address)
      .then((balance) => {
        this.setState({ balance });
      })
      .catch((err) => {
        console.error('could not fetch balance', err);
      });
  }

  render () {
    const { chain, balance, className } = this.props;
    return (
      <div className={ `${styles.container} ${className || ''}` }>
        { this.renderDetails() }
        { this.renderActions() }
      </div>
    );
  }

  renderDetails () {
    const { address, balance, chain, hash } = this.props;

    return (
      <div className={ styles.signDetails }>
        <div className={ styles.address }>
          <Account address={ address } balance={ balance } chain={ chain } />
        </div>
        <div className={ styles.info } title={ hash }>
          <p>Dapp is requesting to sign arbitrary transaction using this account.</p>
          <p><strong>Confirm the transaction only if you trust the app.</strong></p>
        </div>
      </div>
    );
  }

  renderActions () {
    const { address, isFinished, status } = this.props;

    if (isFinished) {
      if (status === 'confirmed') {
        const { chain, hash } = this.props;

        return (
          <div className={ styles.actions }>
            <span className={ styles.isConfirmed }>Confirmed</span>
            <div>
              Transaction hash:
              <TxHashLink chain={ chain } txHash={ hash } className={ styles.txHash } />
            </div>
          </div>
        );
      }

      return (
        <div className={ styles.actions }>
          <span className={ styles.isRejected }>Rejected</span>
        </div>
      );
    }

    return (
      <TransactionPendingForm
        address={ address }
        isSending={ this.props.isSending }
        onConfirm={ this.onConfirm }
        onReject={ this.onReject }
        className={ styles.actions }
        />
    );
  }

  onConfirm = password => {
    const { id } = this.props;
    this.props.onConfirm({ id, password });
  }

  onReject = () => {
    this.props.onReject(this.props.id);
  }

}
