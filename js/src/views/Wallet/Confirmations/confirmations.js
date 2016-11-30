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

import { bytesToHex } from '../../../api/util/format';
import { Container } from '../../../ui';
import { TxRow } from '../../../ui/TxList/txList';

// import styles from '../wallet.css';
import txListStyles from '../../../ui/TxList/txList.css';

export default class WalletDetails extends Component {
  static contextTypes = {
    api: PropTypes.object.isRequired
  };

  static propTypes = {
    address: PropTypes.string.isRequired,
    isTest: PropTypes.bool.isRequired,

    owners: PropTypes.array,
    require: PropTypes.object,
    confirmations: PropTypes.array
  };

  render () {
    return (
      <div>
        <Container title='Pending Confirmations'>
          { this.renderConfirmations() }
        </Container>
      </div>
    );
  }
  renderConfirmations () {
    const { confirmations, address, isTest } = this.props;

    if (!confirmations) {
      return null;
    }

    if (confirmations.length === 0) {
      return (
        <div>
          <p>No transactions needs confirmation right now.</p>
        </div>
      );
    }

    const confirmationsRows = confirmations.map((confirmation) => (
      <TxRow
        key={ confirmation.operation }
        tx={ {
          hash: confirmation.transactionHash,
          blockNumber: confirmation.blockNumber,
          from: address,
          to: confirmation.to,
          value: confirmation.value,
          input: bytesToHex(confirmation.data)
        } }
        address={ address }
        isTest={ isTest }
      />
    ));

    return (
      <table className={ txListStyles.transactions }>
        <tbody>
          { confirmationsRows }
        </tbody>
      </table>
    );
  }
}
