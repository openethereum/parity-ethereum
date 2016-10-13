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

import SignRequest from '../SignRequest';

export default class SignRequestWeb3 extends Component {
  static contextTypes = {
    api: PropTypes.object.isRequired
  }

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
    chain: 'homestead',
    balance: null // avoid required prop loading warning
  }

  componentDidMount () {
    this.fetchChain();
    this.fetchBalance();
  }

  render () {
    const { api } = this.context;
    const { balance, chain } = this.state;
    const { onConfirm, onReject, isSending, isFinished, hash, className, id, status } = this.props;

    const address = api.util.toChecksumAddress(this.props.address);

    return (
      <SignRequest
        address={ address }
        hash={ hash }
        balance={ balance }
        onConfirm={ onConfirm }
        onReject={ onReject }
        isSending={ isSending }
        isFinished={ isFinished }
        id={ id }
        chain={ chain }
        status={ status }
        className={ className }
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

  fetchBalance () {
    const { api } = this.context;
    const { address } = this.props;

    api.eth
      .getBalance(address)
      .then((balance) => {
        this.setState({ balance });
      })
      .catch((error) => {
        console.error('fetchBalance', error);
      });
  }
}
