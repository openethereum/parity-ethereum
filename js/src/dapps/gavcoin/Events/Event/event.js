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

import moment from 'moment';
import React, { Component, PropTypes } from 'react';

import IdentityIcon from '../../IdentityIcon';
import { formatCoins, formatEth, formatHash } from '../../format';

import styles from '../events.css';

const EMPTY_COLUMN = (
  <td></td>
);

export default class Event extends Component {
  static contextTypes = {
    accountsInfo: PropTypes.object.isRequired,
    api: PropTypes.object.isRequired
  }

  static propTypes = {
    event: PropTypes.object,
    value: PropTypes.object,
    price: PropTypes.object,
    fromAddress: PropTypes.string,
    toAddress: PropTypes.string
  }

  state = {
    block: null
  }

  componentDidMount () {
    this.loadBlock();
  }

  render () {
    const { event, fromAddress, toAddress, price, value } = this.props;
    const { block } = this.state;
    const { state, type } = event;
    const cls = `${styles.event} ${styles[state]} ${styles[type.toLowerCase()]}`;

    return (
      <tr className={ cls }>
        { this.renderTimestamp(block) }
        { this.renderType(type) }
        { this.renderValue(value) }
        { this.renderPrice(price) }
        { this.renderAddress(fromAddress) }
        { this.renderAddress(toAddress) }
      </tr>
    );
  }

  renderTimestamp (block) {
    return (
      <td className={ styles.blocknumber }>
        { !block ? ' ' : moment(block.timestamp).fromNow() }
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
    const { accountsInfo } = this.context;
    const account = accountsInfo[address];

    if (account && account.name) {
      return (
        <div className={ styles.name }>
          { account.name }
        </div>
      );
    }

    return (
      <div className={ styles.address }>
        { formatHash(address) }
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

  loadBlock () {
    const { api } = this.context;
    const { event } = this.props;

    if (!event || !event.blockNumber || event.blockNumber.eq(0)) {
      return;
    }

    api.eth
      .getBlockByNumber(event.blockNumber)
      .then((block) => {
        this.setState({ block });
      });
  }
}
