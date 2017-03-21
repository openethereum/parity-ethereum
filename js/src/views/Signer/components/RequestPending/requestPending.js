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

import React, { Component, PropTypes } from 'react';

import TransactionPending from '../TransactionPending';
import SignRequest from '../SignRequest';

export default class RequestPending extends Component {
  static propTypes = {
    className: PropTypes.string,
    date: PropTypes.instanceOf(Date).isRequired,
    focus: PropTypes.bool,
    gasLimit: PropTypes.object.isRequired,
    id: PropTypes.object.isRequired,
    isSending: PropTypes.bool.isRequired,
    netVersion: PropTypes.string.isRequired,
    onConfirm: PropTypes.func.isRequired,
    onReject: PropTypes.func.isRequired,
    origin: PropTypes.object.isRequired,
    payload: PropTypes.oneOfType([
      PropTypes.shape({ sendTransaction: PropTypes.object.isRequired }),
      PropTypes.shape({ sign: PropTypes.object.isRequired }),
      PropTypes.shape({ signTransaction: PropTypes.object.isRequired })
    ]).isRequired,
    signerstore: PropTypes.object.isRequired
  };

  static defaultProps = {
    focus: false,
    isSending: false
  };

  render () {
    const { className, date, focus, gasLimit, id, isSending, netVersion, onReject, payload, signerstore, origin } = this.props;

    if (payload.sign) {
      const { sign } = payload;

      return (
        <SignRequest
          address={ sign.address }
          className={ className }
          focus={ focus }
          data={ sign.data }
          id={ id }
          isFinished={ false }
          isSending={ isSending }
          netVersion={ netVersion }
          onConfirm={ this.onConfirm }
          onReject={ onReject }
          origin={ origin }
          signerstore={ signerstore }
        />
      );
    }

    const transaction = payload.sendTransaction || payload.signTransaction;

    if (transaction) {
      return (
        <TransactionPending
          className={ className }
          date={ date }
          focus={ focus }
          gasLimit={ gasLimit }
          id={ id }
          isSending={ isSending }
          netVersion={ netVersion }
          onConfirm={ this.onConfirm }
          onReject={ onReject }
          origin={ origin }
          signerstore={ signerstore }
          transaction={ transaction }
        />
      );
    }

    console.error('RequestPending: Unknown payload', payload);
    return null;
  }

  onConfirm = (data) => {
    const { onConfirm, payload } = this.props;

    data.payload = payload;
    onConfirm(data);
  };
}
