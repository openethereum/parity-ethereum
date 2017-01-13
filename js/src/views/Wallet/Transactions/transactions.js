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

import { bytesToHex } from '~/api/util/format';
import { Container } from '~/ui';
import TxRow from '~/ui/TxList/TxRow';

import txListStyles from '~/ui/TxList/txList.css';

export default class WalletTransactions extends Component {
  static propTypes = {
    address: PropTypes.string.isRequired,
    isTest: PropTypes.bool.isRequired,
    transactions: PropTypes.array
  };

  static defaultProps = {
    transactions: []
  };

  render () {
    return (
      <div>
        <Container title='Transactions'>
          { this.renderTransactions() }
        </Container>
      </div>
    );
  }
  renderTransactions () {
    const { address, isTest, transactions } = this.props;

    if (!transactions) {
      return null;
    }

    if (transactions.length === 0) {
      return (
        <div>
          <p>No transactions has been sent.</p>
        </div>
      );
    }

    const txRows = transactions.slice(0, 15).map((transaction, index) => {
      const { transactionHash, blockNumber, from, to, value, data } = transaction;

      return (
        <TxRow
          key={ `${transactionHash}_${index}` }
          tx={ {
            hash: transactionHash,
            input: data && bytesToHex(data) || '',
            blockNumber, from, to, value
          } }
          address={ address }
          isTest={ isTest }
        />
      );
    });

    return (
      <table className={ txListStyles.transactions }>
        <tbody>
          { txRows }
        </tbody>
      </table>
    );
  }
}
