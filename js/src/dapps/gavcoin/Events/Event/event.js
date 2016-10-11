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

import IdentityIcon from '../../IdentityIcon';
import { formatBlockNumber, formatCoins, formatEth } from '../../format';

import styles from '../events.css';

const EMPTY_COLUMN = (
  <td></td>
);

export default class Event extends Component {
  static contextTypes = {
    accounts: PropTypes.array.isRequired
  }

  static propTypes = {
    event: PropTypes.object,
    value: PropTypes.object,
    price: PropTypes.object,
    fromAddress: PropTypes.string,
    toAddress: PropTypes.string
  }

  render () {
    const { event, fromAddress, toAddress, price, value } = this.props;
    const { blockNumber, state, type } = event;
    const cls = `${styles.event} ${styles[state]} ${styles[type.toLowerCase()]}`;

    return (
      <tr className={ cls }>
        { this.renderBlockNumber(blockNumber) }
        { this.renderType(type) }
        { this.renderValue(value) }
        { this.renderPrice(price) }
        { this.renderAddress(fromAddress) }
        { this.renderAddress(toAddress) }
      </tr>
    );
  }

  renderBlockNumber (blockNumber) {
    return (
      <td className={ styles.blocknumber }>
        { formatBlockNumber(blockNumber) }
      </td>
    );
  }

  renderAddress (address) {
    if (!address) {
      return EMPTY_COLUMN;
    }

    return (
      <td className={ styles.account }>
        <IdentityIcon address={ address } />
        { this.renderAddressName(address) }
      </td>
    );
  }

  renderAddressName (address) {
    const { accounts } = this.context;
    const account = accounts.find((_account) => _account.address === address);

    if (account) {
      return (
        <div className={ styles.name }>
          { account.name }
        </div>
      );
    }

    return (
      <div className={ styles.address }>
        { address }
      </div>
    );
  }

  renderPrice (price) {
    if (!price) {
      return EMPTY_COLUMN;
    }

    return (
      <td className={ styles.ethvalue }>
        { formatEth(price) }<small> ETH</small>
      </td>
    );
  }

  renderValue (value) {
    if (!value) {
      return EMPTY_COLUMN;
    }

    return (
      <td className={ styles.gavvalue }>
        { formatCoins(value) }<small> GAV</small>
      </td>
    );
  }

  renderType (type) {
    return (
      <td className={ styles.type }>
        { type }
      </td>
    );
  }
}
