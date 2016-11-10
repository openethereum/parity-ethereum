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
import qs from 'querystring';

import TxHash from '../../../ui/TxHash';
import waitForConfirmations from '../wait-for-confirmations';

import styles from './sendRequest.css';

const postToVerificationServer = (query) => {
  query = qs.stringify(query);
  return fetch('https://sms-verification.parity.io/?' + query, {
    method: 'POST', mode: 'cors', cache: 'no-store'
  })
  .then((res) => {
    return res.json().then((data) => {
      if (res.ok) {
        return data.message;
      }
      throw new Error(data.message || 'unknown error');
    });
  });
};

export default class SendRequest extends Component {
  static contextTypes = {
    api: PropTypes.object.isRequired
  }

  static propTypes = {
    account: PropTypes.string.isRequired,
    contract: PropTypes.object.isRequired,
    data: PropTypes.object.isRequired,
    onData: PropTypes.func.isRequired,
    onSuccess: PropTypes.func.isRequired,
    onError: PropTypes.func.isRequired,
    nextStep: PropTypes.func.isRequired
  }

  state = {
    init: true,
    step: 'init'
  };

  componentWillMount () {
    const { init } = this.state;
    if (init) {
      this.send();
      this.setState({ init: false });
    }
  }

  render () {
    const { step } = this.state;

    if (step === 'error') {
      return (<p>{ this.state.error }</p>);
    }

    if (step === 'pending') {
      return (<p>A verification request will be sent to the contract. Please authorize this using the Parity Signer.</p>);
    }

    if (step === 'posted') {
      return (
        <div className={ styles.centered }>
          <TxHash hash={ this.props.data.txHash } maxConfirmations={ 3 } />
          <p>Please keep this window open.</p>
        </div>);
    }

    if (step === 'mined') {
      return (<p>Requesting an SMS from the Parity server.</p>);
    }

    return null;
  }

  send = () => {
    const { api } = this.context;
    const { account, contract, onData, onError, onSuccess, nextStep } = this.props;
    const { fee, number, hasRequested } = this.props.data;

    const request = contract.functions.find((fn) => fn.name === 'request');
    const options = { from: account, value: fee.toString() };

    let chain = Promise.resolve();
    if (!hasRequested) {
      chain = request.estimateGas(options, [])
        .then((gas) => {
          options.gas = gas.mul(1.2).toFixed(0);
          // TODO: show message
          this.setState({ step: 'pending' });
          return request.postTransaction(options, []);
        })
        .then((handle) => {
          // TODO: The "request rejected" error doesn't have any property to
          // distinguish it from other errors, so we can't give a meaningful error here.
          return api.pollMethod('parity_checkRequest', handle);
        })
        .then((txHash) => {
          onData({ txHash: txHash });
          this.setState({ step: 'posted' });
          return waitForConfirmations(api, txHash, 3);
        });
    }

    chain
      .then(() => {
        this.setState({ step: 'mined' });
        return postToVerificationServer({ number, address: account });
      })
      .then(() => {
        this.setState({ step: 'sms-sent' });
        onSuccess();
        nextStep();
      })
      .catch((err) => {
        console.error('failed to request sms verification', err);
        onError(err);
        this.setState({
          step: 'error',
          error: 'Failed to request a confirmation SMS: ' + err.message
        });
        // TODO: show message in SnackBar
      });
  }
}
