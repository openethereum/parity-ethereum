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

import { IdentityIcon, IdentityName, MethodDecoding } from '../../../../ui';

import styles from '../transactions.css';

export default class Transaction extends Component {
  static contextTypes = {
    api: PropTypes.object.isRequired
  }

  static propTypes = {
    transaction: PropTypes.object.isRequired,
    address: PropTypes.string.isRequired,
    isTest: PropTypes.bool.isRequired
  }

  state = {
    info: null,
    isContract: false,
    isReceived: false
  }

  componentDidMount () {
    const { address, transaction } = this.props;

    this.lookup(address, transaction);
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
    const { address } = this.props;
    const { info } = this.state;

    if (!info) {
      return null;
    }

    return (
      <MethodDecoding
        historic
        address={ address }
        transaction={ info } />
    );
  }

  renderTransaction () {
    const { transaction, isTest } = this.props;

    const prefix = `https://${isTest ? 'testnet.' : ''}etherscan.io/`;
    const hashLink = `${prefix}tx/${transaction.hash}`;

    return (
      <td className={ styles.transaction }>
        { this.renderEtherValue() }
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
    const eslink = address ? (
      <a
        href={ `${prefix}address/${address}` }
        target='_blank'
        className={ styles.link }>
        <IdentityName address={ address } shorten />
      </a>
    ) : 'DEPLOY';

    return (
      <td className={ styles.address }>
        <div className={ styles.center }>
          <IdentityIcon
            center
            className={ styles.icon }
            address={ address } />
        </div>
        <div className={ styles.center }>
          { eslink }
        </div>
      </td>
    );
  }

  renderEtherValue () {
    const { api } = this.context;
    const { info } = this.state;

    if (!info) {
      return null;
    }

    const value = api.util.fromWei(info.value);

    if (value.eq(0)) {
      return <div className={ styles.value }>{ ' ' }</div>;
    }

    return (
      <div className={ styles.value }>
        { value.toFormat(5) }<small>ETH</small>
      </div>
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

  lookup (address, transaction) {
    const { api } = this.context;
    const { info } = this.state;

    if (info) {
      return;
    }

    this.setState({ isReceived: address === transaction.to });

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
