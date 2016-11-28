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
import SignRequest from '../SignRequest';

export default class RequestFinished extends Component {
  static propTypes = {
    id: PropTypes.object.isRequired,
    result: PropTypes.any.isRequired,
    date: PropTypes.instanceOf(Date).isRequired,
    payload: PropTypes.oneOfType([
      PropTypes.shape({ signTransaction: PropTypes.object.isRequired }),
      PropTypes.shape({ sendTransaction: PropTypes.object.isRequired }),
      PropTypes.shape({ sign: PropTypes.object.isRequired })
    ]).isRequired,
    msg: PropTypes.string,
    status: PropTypes.string,
    error: PropTypes.string,
    className: PropTypes.string,
    isTest: PropTypes.bool.isRequired,
    store: PropTypes.object.isRequired
  }

  render () {
    const { payload, id, result, msg, status, error, date, className, isTest, store } = this.props;

    if (payload.sign) {
      const { sign } = payload;

      return (
        <SignRequest
          className={ className }
          isFinished
          id={ id }
          address={ sign.address }
          hash={ sign.hash }
          msg={ msg }
          status={ status }
          error={ error }
          isTest={ isTest }
          store={ store }
          />
      );
    }

    if (payload.sendTransaction || payload.signTransaction) {
      const transaction = payload.sendTransaction || payload.signTransaction;

      return (
        <TransactionFinished
          className={ className }
          txHash={ result }
          id={ id }
          gasPrice={ transaction.gasPrice }
          gas={ transaction.gas }
          from={ transaction.from }
          to={ transaction.to }
          value={ transaction.value }
          msg={ msg }
          date={ date }
          status={ status }
          error={ error }
          isTest={ isTest }
          store={ store }
        />
      );
    }

    // Unknown payload
    return null;
  }
}
