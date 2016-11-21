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
import ShortenedHash from '../../../../ui/ShortenedHash';
import { txLink, addressLink } from '../../../../3rdparty/etherscan/links';

import styles from '../transactions.css';

export default class Transaction extends Component {
  static contextTypes = {
    api: PropTypes.object.isRequired
  }

  static propTypes = {
    address: PropTypes.string.isRequired,
    isTest: PropTypes.bool.isRequired,
    transaction: PropTypes.object.isRequired
  }

  state = {
    isContract: false,
    isReceived: false,
    transaction: null,
    block: null
  }

  componentDidMount () {
    this.lookup();
  }

  render () {
    const { block } = this.state;
    const { transaction } = this.props;

    return (
      <tr>
        <td className={ styles.timestamp }>
          <div>{ this.formatBlockTimestamp(block) }</div>
          <div>{ this.formatNumber(transaction.blockNumber) }</div>
        </td>
        { this.renderAddress(transaction.from) }
        { this.renderTransaction() }
        { this.renderAddress(transaction.to) }
        <td className={ styles.method }>
          { this.renderMethod() }
        </td>
      </tr>
    );
  }

  renderMethod () {
    const { address } = this.props;
    const { transaction } = this.state;

    if (!transaction) {
      return null;
    }

    return (
      <MethodDecoding
        historic
        address={ address }
        transaction={ transaction } />
    );
  }

  renderTransaction () {
    const { isTest } = this.props;
    const { transaction } = this.props;

    return (
      <td className={ styles.transaction }>
        { this.renderEtherValue() }
        <div>⇒</div>
        <div>
          <a
            className={ styles.link }
            href={ txLink(transaction.hash, isTest) }
            target='_blank'
          >
            <ShortenedHash data={ transaction.hash } />
          </a>
        </div>
      </td>
    );
  }

  renderAddress (address) {
    const { isTest } = this.props;

    const eslink = address ? (
      <a
        href={ addressLink(address, isTest) }
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
    const { transaction } = this.state;

    if (!transaction) {
      return null;
    }

    const value = api.util.fromWei(transaction.value);

    if (value.eq(0)) {
      return <div className={ styles.value }>{ ' ' }</div>;
    }

    return (
      <div className={ styles.value }>
        { value.toFormat(5) }<small>ETH</small>
      </div>
    );
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

  lookup () {
    const { api } = this.context;
    const { transaction, address } = this.props;

    this.setState({ isReceived: address === transaction.to });

    Promise
      .all([
        api.eth.getBlockByNumber(transaction.blockNumber),
        api.eth.getTransactionByHash(transaction.hash)
      ])
      .then(([block, transaction]) => {
        this.setState({ block, transaction });
      })
      .catch((error) => {
        console.warn('lookup', error);
      });
  }
}
