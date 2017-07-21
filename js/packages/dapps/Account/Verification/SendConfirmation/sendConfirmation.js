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

import React, { Component } from 'react';
import PropTypes from 'prop-types';
import { FormattedMessage } from 'react-intl';

import { nullableProptype } from '@parity/shared/util/proptypes';
import TxHash from '@parity/ui/TxHash';

import { POSTING_CONFIRMATION, POSTED_CONFIRMATION } from '../store';

import styles from './sendConfirmation.css';

export default class SendConfirmation extends Component {
  static propTypes = {
    step: PropTypes.any.isRequired,
    tx: nullableProptype(PropTypes.any.isRequired)
  }

  render () {
    const { step, tx } = this.props;

    if (step === POSTING_CONFIRMATION) {
      return (
        <p>
          <FormattedMessage
            id='verification.confirmation.authorise'
            defaultMessage='The verification code will be sent to the contract. Please authorize this using the Parity Signer.'
          />
        </p>);
    }

    if (step === POSTED_CONFIRMATION) {
      return (
        <div className={ styles.centered }>
          <TxHash
            hash={ tx }
            maxConfirmations={ 2 }
          />
          <p>
            <FormattedMessage
              id='verification.confirmation.windowOpen'
              defaultMessage='Please keep this window open.'
            />
          </p>
        </div>
      );
    }

    return null;
  }
}
