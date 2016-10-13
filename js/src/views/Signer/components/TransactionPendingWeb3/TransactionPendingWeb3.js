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

import TransactionPending from '../TransactionPending';

export default class TransactionPendingWeb3 extends Component {
  static contextTypes = {
    api: PropTypes.object.isRequired
  };

  static propTypes = {
    id: PropTypes.object.isRequired,
    from: PropTypes.string.isRequired,
    value: PropTypes.object.isRequired, // wei hex
    gasPrice: PropTypes.object.isRequired, // wei hex
    gas: PropTypes.object.isRequired, // hex
    onConfirm: PropTypes.func.isRequired,
    onReject: PropTypes.func.isRequired,
    isSending: PropTypes.bool.isRequired,
    date: PropTypes.instanceOf(Date).isRequired,
    to: PropTypes.string, // undefined if it's a contract
    data: PropTypes.string, // hex
    nonce: PropTypes.number,
    className: PropTypes.string
  };

  state = {
    chain: 'homestead',
    fromBalance: null, // avoid required prop loading warning
    toBalance: null // avoid required prop loading warning in case there's a to address
  }

  componentDidMount () {
    this.fetchChain();
    this.fetchBalances();
  }

  render () {
    const { fromBalance, toBalance, chain } = this.state;
    const { from, to, date } = this.props;

    return (
      <TransactionPending
        { ...this.props }
        from={ from }
        to={ to }
        fromBalance={ fromBalance }
        toBalance={ toBalance }
        chain={ chain }
        date={ date }
      />
    );
  }

  fetchChain () {
    const { api } = this.context;

    api.ethcore
      .netChain()
      .then((chain) => {
        this.setState({ chain });
      })
      .catch((error) => {
        console.error('fetchChain', error);
      });
  }

  fetchBalances () {
    const { from, to } = this.props;
    this.fetchBalance(from, 'from');

    if (!to) {
      return;
    }

    this.fetchBalance(to, 'to');
  }

  fetchBalance (address, owner) {
    const { api } = this.context;

    api.eth
      .getBalance(address)
      .then((balance) => {
        this.setState({ [owner + 'Balance']: balance });
      })
      .catch((error) => {
        console.error('fetchBalance', error);
      });
  }
}
