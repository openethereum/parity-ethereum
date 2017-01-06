// Copyright 2015, 2016 Parity Technologies (UK) Ltd.
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
import { FormattedMessage } from 'react-intl';
import BigNumber from 'bignumber.js';

import AccountSummary from '~/views/Accounts/Summary';

export default class Summary extends Component {

  static propTypes = {
    account: PropTypes.object.isRequired
  };

  render () {
    const { account, ...props } = this.props;

    const description = this.getDescription(account.meta);

    return (
      <AccountSummary
        account={ account }
        description={ description }
        { ...props }
      />
    );
  }

  getDescription (meta = {}) {
    const { blockNumber } = meta;

    if (!blockNumber) {
      return null;
    }

    const formattedBlockNumber = (new BigNumber(blockNumber)).toFormat();

    return (
      <FormattedMessage
        id='contract.summary.minedBlock'
        defaultMessage='Mined at block #{blockNumber}'
        values={ {
          blockNumber: formattedBlockNumber
        } }
      />
    );
  }
}
