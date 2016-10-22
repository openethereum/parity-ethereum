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
import moment from 'moment';
import React, { Component, PropTypes } from 'react';
import { connect } from 'react-redux';
import { bindActionCreators } from 'redux';

import { fetchBlock, fetchTransaction } from '../../../../redux/providers/blockchainActions';

import styles from '../../contract.css';

class Event extends Component {
  static contextTypes = {
    api: PropTypes.object.isRequired
  }

  static propTypes = {
    event: PropTypes.object.isRequired,
    blocks: PropTypes.object,
    transactions: PropTypes.object,
    isTest: PropTypes.bool,
    fetchBlock: PropTypes.func.isRequired,
    fetchTransaction: PropTypes.func.isRequired
  }

  componentDidMount () {
    this.retrieveTransaction();
  }

  render () {
    const { event, blocks, transactions, isTest } = this.props;

    const block = blocks[event.blockNumber.toString()];
    const transaction = transactions[event.transactionHash] || {};
    const classes = `${styles.event} ${styles[event.state]}`;
    const url = `https://${isTest ? 'testnet.' : ''}etherscan.io/tx/${event.transactionHash}`;
    const keys = Object.keys(event.params).map((key, index) => {
      return <div className={ styles.key } key={ `${event.key}_key_${index}` }>{ key }</div>;
    });
    const values = Object.values(event.params).map((param, index) => {
      return (
        <div className={ styles.value } key={ `${event.key}_val_${index}` }>
          { this.renderParam(param) }
        </div>
      );
    });

    return (
      <tr className={ classes }>
        <td className={ styles.timestamp }>
          <div>{ event.state === 'pending' ? 'pending' : this.formatBlockTimestamp(block) }</div>
          <div>{ this.formatNumber(transaction.blockNumber) }</div>
        </td>
        <td className={ styles.txhash }>
          <div>{ transaction.from }</div>
          <a href={ url } target='_blank'>{ event.transactionHash }</a>
        </td>
        <td>
          <div>{ event.type } =></div>
          { keys }
        </td>
        <td>
          <div>&nbsp;</div>
          { values }
        </td>
      </tr>
    );
  }

  renderParam (param) {
    const { api } = this.context;

    if (api.util.isInstanceOf(param.value, BigNumber)) {
      return param.value.toFormat(0);
    } else if (api.util.isArray(param.value)) {
      return api.util.bytesToHex(param.value);
    }

    return param.value.toString();
  }

  formatBlockTimestamp (block) {
    if (!block) {
      return null;
    }

    return moment(block.timestamp).fromNow();
  }

  formatNumber (number) {
    if (!number) {
      return null;
    }

    return new BigNumber(number).toFormat();
  }

  retrieveTransaction () {
    const { event, fetchBlock, fetchTransaction } = this.props;

    fetchBlock(event.blockNumber);
    fetchTransaction(event.transactionHash);
  }
}

function mapStateToProps (state) {
  const { isTest } = state.nodeStatus;
  const { blocks, transactions } = state.blockchain;

  return {
    isTest,
    blocks,
    transactions
  };
}

function mapDispatchToProps (dispatch) {
  return bindActionCreators({
    fetchBlock, fetchTransaction
  }, dispatch);
}

export default connect(
  mapStateToProps,
  mapDispatchToProps
)(Event);
