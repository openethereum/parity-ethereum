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
import { connect } from 'react-redux';
import { bindActionCreators } from 'redux';
import { observer } from 'mobx-react';

import { txLink, addressLink } from '../../3rdparty/etherscan/links';

import IdentityIcon from '../IdentityIcon';
import IdentityName from '../IdentityName';
import MethodDecoding from '../MethodDecoding';
import Store from './store';

import styles from './txList.css';

@observer
class TxList extends Component {
  static contextTypes = {
    api: PropTypes.object.isRequired
  }

  static propTypes = {
    address: PropTypes.string.isRequired,
    hashes: PropTypes.array.isRequired,
    isTest: PropTypes.bool.isRequired
  }

  store = new Store(this.context.api);

  componentWillMount () {
    this.store.loadTransactions(this.props.hashes);
  }

  componentWillReceiveProps (newProps) {
    this.store.loadTransactions(newProps.hashes);
  }

  render () {
    return (
      <table className={ styles.transactions }>
        <tbody>
          { this.renderRows() }
        </tbody>
      </table>
    );
  }

  renderRows () {
    const { address, isTest } = this.props;

    return this.store.transactions.map((transaction) => {
      return (
        <tr key={ transaction.hash }>
          <td className={ styles.timestamp }>
            <div>{ this.formatBlockTimestamp(transaction.blockNumber) }</div>
            <div>{ transaction.blockNumber.toFormat() }</div>
          </td>
          { this.renderAddress(transaction.from) }
          <td className={ styles.transaction }>
            { this.renderEtherValue(transaction.value) }
            <div>⇒</div>
            <div>
              <a
                className={ styles.link }
                href={ txLink(transaction.hash, isTest) }
                target='_blank'>
                { `${transaction.hash.substr(2, 6)}...${transaction.hash.slice(-6)}` }
              </a>
            </div>
          </td>
          { this.renderAddress(transaction.to) }
          <td className={ styles.method }>
            <MethodDecoding
              historic
              address={ address }
              transaction={ transaction } />
          </td>
        </tr>
      );
    });
  }

  renderAddress (address) {
    const { isTest } = this.props;

    let esLink = null;
    if (address) {
      esLink = (
        <a
          href={ addressLink(address, isTest) }
          target='_blank'
          className={ styles.link }>
          <IdentityName address={ address } shorten />
        </a>
      );
    }

    return (
      <td className={ styles.address }>
        <div className={ styles.center }>
          <IdentityIcon
            center
            className={ styles.icon }
            address={ address } />
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

  formatBlockTimestamp (blockNumber) {
    const block = this.store.blocks[blockNumber.toNumber()];

    if (!block) {
      return null;
    }

    return moment(block.timestamp).fromNow();
  }
}

function mapStateToProps (state) {
  const { isTest } = state.nodeStatus;

  return {
    isTest
  };
}

function mapDispatchToProps (dispatch) {
  return bindActionCreators({}, dispatch);
}

export default connect(
  mapStateToProps,
  mapDispatchToProps
)(TxList);
