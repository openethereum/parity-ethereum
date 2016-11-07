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
import { connect } from 'react-redux';
import { bindActionCreators } from 'redux';

import { fetchBlock, fetchTransaction } from '../../../../redux/providers/blockchainActions';

import { IdentityIcon, IdentityName, MethodDecoding } from '../../../../ui';
import { txLink, addressLink } from '../../../../3rdparty/etherscan/links';

import styles from '../transactions.css';

class Transaction extends Component {
  static contextTypes = {
    api: PropTypes.object.isRequired
  }

  static propTypes = {
    transaction: PropTypes.object.isRequired,
    address: PropTypes.string.isRequired,
    isTest: PropTypes.bool.isRequired,

    fetchBlock: PropTypes.func.isRequired,
    fetchTransaction: PropTypes.func.isRequired,

    block: PropTypes.object,
    transactionInfo: PropTypes.object
  }

  state = {
    isContract: false,
    isReceived: false
  }

  componentDidMount () {
    const { address, transaction } = this.props;

    this.lookup(address, transaction);
  }

  render () {
    const { block, transaction } = this.props;

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
    const { address, transactionInfo } = this.props;

    if (!transactionInfo) {
      return null;
    }

    return (
      <MethodDecoding
        historic
        address={ address }
        transaction={ transactionInfo } />
    );
  }

  renderTransaction () {
    const { transaction, isTest } = this.props;

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
            { this.formatHash(transaction.hash) }
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
    const { transactionInfo } = this.props;

    if (!transactionInfo) {
      return null;
    }

    const value = api.util.fromWei(transactionInfo.value);

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
    const { transactionInfo } = this.props;

    if (transactionInfo) {
      return;
    }

    this.setState({ isReceived: address === transaction.to });

    const { fetchBlock, fetchTransaction } = this.props;
    const { blockNumber, hash } = transaction;

    fetchBlock(blockNumber);
    fetchTransaction(hash);
  }
}

function mapStateToProps () {
  return {};
}

function mapDispatchToProps (dispatch) {
  return bindActionCreators({
    fetchBlock, fetchTransaction
  }, dispatch);
}

export default connect(
  mapStateToProps,
  mapDispatchToProps
)(Transaction);
