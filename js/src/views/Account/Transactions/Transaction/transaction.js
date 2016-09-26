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
    address: PropTypes.string.isRequired,
    accounts: PropTypes.object,
    contacts: PropTypes.object,
    tokens: PropTypes.object,
    isTest: PropTypes.bool.isRequired
  }

  state = {
    info: null,
    isContract: false,
    isReceived: this.props.address === this.props.transaction.to
  }

  componentDidMount () {
    const { transaction } = this.props;

    this.lookup(transaction);
  }

  render () {
    const { transaction, isTest } = this.props;
    const { block } = this.state;

    const prefix = `https://${isTest ? 'testnet.' : ''}etherscan.io/`;

    return (
      <tr>
        <td className={ styles.timestamp }>
          <div>{ this.formatBlockTimestamp(block) }</div>
          <div>{ this.formatNumber(transaction.blockNumber) }</div>
        </td>
        { this.renderAddress(prefix, transaction.from) }
        { this.renderTransaction() }
        { this.renderAddress(prefix, transaction.to) }
        <td className={ styles.method }>
          { this.renderMethod() }
        </td>
      </tr>
    );
  }

  renderMethod () {
    const { info, isContract, isReceived } = this.state;

    if (!info) {
      return null;
    }

    return (
      <MethodDecoding
        historic
        isContract={ isContract }
        isReceived={ isReceived }
        transaction={ info } />
    );
  }

  renderTransaction () {
    const { transaction, isTest } = this.props;

    const prefix = `https://${isTest ? 'testnet.' : ''}etherscan.io/`;
    const hashLink = `${prefix}tx/${transaction.hash}`;
    const { value, token } = this.formatEther(transaction.value);

    return (
      <td className={ styles.transaction }>
        <div className={ styles.value }>
          { value }{ token }
        </div>
        <div>⇒</div>
        <div>
          <a href={ hashLink } target='_blank' className={ styles.link }>
            { this.formatHash(transaction.hash) }
          </a>
        </div>
      </td>
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
      <td className={ styles.address }>
        <div className={ styles.center }>
          <IdentityIcon
            center
            className={ styles.icon }
            tokens={ tokens }
            address={ address } />
        </div>
        <div className={ styles.center }>
          <a
            href={ link }
            target='_blank'
            className={ styles.link }>
            { name }
          </a>
        </div>
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

    if (ether.eq(0)) {
      return { value: null, token: null };
    }

    return {
      value: `${ether.toFormat(5)}`,
      token: <small>ΞTH</small>
    };
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

        if (!transaction.to) {
          return;
        }

        return api.eth
          .getCode(transaction.to)
          .then((code) => {
            this.setState({ block, info, isContract: code !== '0x' });
          });
      })
      .catch((error) => {
        console.error('lookup', error);
      });
  }
}
