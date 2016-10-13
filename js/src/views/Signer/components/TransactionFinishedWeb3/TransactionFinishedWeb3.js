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

import TransactionFinished from '../TransactionFinished';

export default class TransactionFinishedWeb3 extends Component {
  static contextTypes = {
    api: PropTypes.object.isRequired
  }

  static propTypes = {
    from: PropTypes.string.isRequired,
    to: PropTypes.string // undefined if it's a contract
  }

  state = {
    chain: 'homestead'
  }

  componentDidMount () {
    this.fetchChain();
    this.fetchBalances();
  }

  render () {
    const { fromBalance, toBalance, chain } = this.state;
    const { from, to } = this.props;

    return (
      <TransactionFinished
        { ...this.props }
        from={ from }
        fromBalance={ fromBalance }
        to={ to }
        toBalance={ toBalance }
        chain={ chain }
        />
    );
  }

  fetchChain () {
    const { api } = this.context;

    api.ethcore
      .getNetChain()
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
