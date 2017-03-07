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
import { connect } from 'react-redux';
import { Link } from 'react-router';

import { txLink } from '~/3rdparty/etherscan/links';

import IdentityIcon from '../../IdentityIcon';
import IdentityName from '../../IdentityName';
import MethodDecoding from '../../MethodDecoding';

import styles from '../txList.css';

class TxRow extends Component {
  static contextTypes = {
    api: PropTypes.object.isRequired
  };

  static propTypes = {
    accountAddresses: PropTypes.array.isRequired,
    address: PropTypes.string.isRequired,
    netVersion: PropTypes.string.isRequired,
    tx: PropTypes.object.isRequired,

    block: PropTypes.object,
    className: PropTypes.string,
    historic: PropTypes.bool
  };

  static defaultProps = {
    historic: true
  };

  render () {
    const { address, className, historic, netVersion, tx } = this.props;

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
              href={ txLink(tx.hash, false, netVersion) }
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
    let esLink = null;

    if (address) {
      esLink = (
        <Link
          activeClassName={ styles.currentLink }
          className={ styles.link }
          to={ this.addressLink(address) }
        >
          <IdentityName
            address={ address }
            shorten
          />
        </Link>
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

  addressLink (address) {
    const { accountAddresses } = this.props;
    const isAccount = accountAddresses.includes(address);

    if (isAccount) {
      return `/accounts/${address}`;
    }

    return `/addresses/${address}`;
  }
}

function mapStateToProps (initState) {
  const { accounts } = initState.personal;
  const accountAddresses = Object.keys(accounts);

  return (state) => {
    const { netVersion } = state.nodeStatus;

    return {
      accountAddresses,
      netVersion
    };
  };
}

export default connect(
  mapStateToProps,
  null
)(TxRow);
