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

import BigNumber from 'bignumber.js';
import React, { Component, PropTypes } from 'react';
import moment from 'moment';

import { IdentityIcon, MethodDecoding } from '../../../../ui';

import styles from '../transactions.css';

export default class Transaction extends Component {
  static contextTypes = {
    api: PropTypes.object.isRequired
  }

  static propTypes = {
    transaction: PropTypes.object.isRequired,
    accounts: PropTypes.object,
    contacts: PropTypes.object,
    tokens: PropTypes.object,
    isTest: PropTypes.bool.isRequired
  }

  state = {
    info: null
  }

  componentDidMount () {
    const { transaction } = this.props;

    this.lookup(transaction);
  }

  render () {
    const { transaction, isTest } = this.props;
    const { block } = this.state;

    const prefix = `https://${isTest ? 'testnet.' : ''}etherscan.io/`;
    const hashLink = `${prefix}tx/${transaction.hash}`;
    const value = this.formatEther(transaction.value);
    const token = value ? 'ΞTH' : null;

    return (
      <tr>
        <td className={ styles.right }>
          { this.formatBlockTimestamp(block) }
        </td>
        <td className={ styles.right }>
          { this.formatNumber(transaction.blockNumber) }
        </td>
        { this.renderAddress(prefix, transaction.from) }
        { this.renderAddress(prefix, transaction.to) }
        <td className={ styles.center }>
          <a href={ hashLink } target='_blank' className={ styles.link }>
            { this.formatHash(transaction.hash) }
          </a>
        </td>
        <td className={ styles.value }>
          { this.formatEther(transaction.value) }<small> { token }</small>
        </td>
        <td className={ styles.method }>
          { this.renderMethod() }
        </td>
      </tr>
    );
  }

  renderMethod () {
    const { info } = this.state;

    if (!info) {
      return null;
    }

    return (
      <MethodDecoding input={ info.input } />
    );
  }

  renderAddress (prefix, address) {
    const { accounts, contacts, tokens } = this.props;

    if (!address && !address.length) {
      return (
        <td className={ styles.left } />
      );
    }

    const account = (accounts || {})[address] || (contacts || {})[address] || (tokens || {})[address];
    const link = `${prefix}address/${address}`;
    const name = account
      ? account.name.toUpperCase()
      : this.formatHash(address);

    return (
      <td className={ styles.left }>
        <IdentityIcon
          inline center
          tokens={ tokens }
          address={ address } />
        <a
          href={ link }
          target='_blank'
          className={ styles.link }>
          { name }
        </a>
      </td>
    );
  }

  formatHash (hash) {
    if (!hash || hash.length <= 16) {
      return hash;
    }

    return `${hash.substr(2, 6)}...${hash.slice(-6)}`;
  }

  formatNumber (number) {
    return new BigNumber(number).toFormat();
  }

  formatBlockTimestamp (block) {
    if (!block) {
      return null;
    }

    return moment(block.timestamp).fromNow();
  }

  formatEther (value) {
    const { api } = this.context;
    const ether = api.util.fromWei(value);

    if (ether.gt(0)) {
      return `${ether.toFormat(5)}`;
    }

    return null;
  }

  lookup (transaction) {
    const { api } = this.context;
    const { info } = this.state;

    if (info) {
      return;
    }

    Promise
      .all([
        api.eth.getBlockByNumber(transaction.blockNumber),
        api.eth.getTransactionByHash(transaction.hash)
      ])
      .then(([block, info]) => {
        this.setState({ block, info });
      })
      .catch((error) => {
        console.error('lookup', error);
      });
  }
}
