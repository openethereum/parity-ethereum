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

import moment from 'moment';
import React, { Component, PropTypes } from 'react';

import { txLink, addressLink } from '~/3rdparty/etherscan/links';

import IdentityIcon from '../../IdentityIcon';
import IdentityName from '../../IdentityName';
import MethodDecoding from '../../MethodDecoding';

import styles from '../txList.css';

export default class TxRow extends Component {
  static contextTypes = {
    api: PropTypes.object.isRequired
  };

  static propTypes = {
    tx: PropTypes.object.isRequired,
    address: PropTypes.string.isRequired,
    isTest: PropTypes.bool.isRequired,

    block: PropTypes.object,
    historic: PropTypes.bool,
    className: PropTypes.string
  };

  static defaultProps = {
    historic: true
  };

  render () {
    const { tx, address, isTest, historic, className } = this.props;

    return (
      <tr className={ className || '' }>
        { this.renderBlockNumber(tx.blockNumber) }
        { this.renderAddress(tx.from) }
        <td className={ styles.transaction }>
          { this.renderEtherValue(tx.value) }
          <div>⇒</div>
          <div>
            <a
              className={ styles.link }
              href={ txLink(tx.hash, isTest) }
              target='_blank'
            >
              { `${tx.hash.substr(2, 6)}...${tx.hash.slice(-6)}` }
            </a>
          </div>
        </td>
        { this.renderAddress(tx.to) }
        <td className={ styles.method }>
          <MethodDecoding
            historic={ historic }
            address={ address }
            transaction={ tx }
          />
        </td>
      </tr>
    );
  }

  renderAddress (address) {
    const { isTest } = this.props;

    let esLink = null;

    if (address) {
      esLink = (
        <a
          href={ addressLink(address, isTest) }
          target='_blank'
          className={ styles.link }
        >
          <IdentityName
            address={ address }
            shorten
          />
        </a>
      );
    }

    return (
      <td className={ styles.address }>
        <div className={ styles.center }>
          <IdentityIcon
            center
            className={ styles.icon }
            address={ address }
          />
        </div>
        <div className={ styles.center }>
          { esLink || 'DEPLOY' }
        </div>
      </td>
    );
  }

  renderEtherValue (_value) {
    const { api } = this.context;
    const value = api.util.fromWei(_value);

    if (value.eq(0)) {
      return <div className={ styles.value }>{ ' ' }</div>;
    }

    return (
      <div className={ styles.value }>
        { value.toFormat(5) }<small>ETH</small>
      </div>
    );
  }

  renderBlockNumber (_blockNumber) {
    const { block } = this.props;
    const blockNumber = _blockNumber.toNumber();

    return (
      <td className={ styles.timestamp }>
        <div>{ blockNumber && block ? moment(block.timestamp).fromNow() : null }</div>
        <div>{ blockNumber ? _blockNumber.toFormat() : 'Pending' }</div>
      </td>
    );
  }
}
