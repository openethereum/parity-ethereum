// Copyright 2015, 2016 Parity Technologies (UK) Ltd.
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

import React, { Component, PropTypes } from 'react';
import { connect } from 'react-redux';
import { bindActionCreators } from 'redux';

import etherscan from '~/3rdparty/etherscan';
import { Container, TxList, Loading } from '~/ui';

import styles from './transactions.css';

class Transactions extends Component {
  static contextTypes = {
    api: PropTypes.object.isRequired
  }

  static propTypes = {
    address: PropTypes.string.isRequired,
    isTest: PropTypes.bool,
    traceMode: PropTypes.bool
  }

  state = {
    hashes: [],
    loading: true,
    callInfo: {}
  }

  componentDidMount () {
    this.getTransactions(this.props);
  }

  componentWillReceiveProps (newProps) {
    if (this.props.traceMode === undefined && newProps.traceMode !== undefined) {
      this.getTransactions(newProps);
      return;
    }

    const hasChanged = [ 'isTest', 'address' ]
      .map(key => newProps[key] !== this.props[key])
      .reduce((truth, keyTruth) => truth || keyTruth, false);

    if (hasChanged) {
      this.getTransactions(newProps);
    }
  }

  render () {
    return (
      <Container title='transactions'>
        { this.renderTransactionList() }
        { this.renderEtherscanFooter() }
      </Container>
    );
  }

  renderTransactionList () {
    const { address } = this.props;
    const { hashes, loading } = this.state;

    if (loading) {
      return (
        <Loading />
      );
    }

    return (
      <TxList
        address={ address }
        hashes={ hashes }
      />
    );
  }

  renderEtherscanFooter () {
    const { traceMode } = this.props;

    if (traceMode) {
      return null;
    }

    return (
      <div className={ styles.etherscan }>
        Transaction list powered by <a href='https://etherscan.io/' target='_blank'>etherscan.io</a>
      </div>
    );
  }

  getTransactions = (props) => {
    const { isTest, address, traceMode } = props;

    // Don't fetch the transactions if we don't know in which
    // network we are yet...
    if (isTest === undefined) {
      return;
    }

    return this
      .fetchTransactions(isTest, address, traceMode)
      .then((transactions) => {
        this.setState({
          hashes: transactions.map((transaction) => transaction.hash),
          loading: false
        });
      });
  }

  fetchTransactions = (isTest, address, traceMode) => {
    // if (traceMode) {
    //   return this.fetchTraceTransactions(address);
    // }

    return this.fetchEtherscanTransactions(isTest, address);
  }

  fetchEtherscanTransactions = (isTest, address) => {
    return etherscan.account
      .transactions(address, 0, isTest)
      .catch((error) => {
        console.error('getTransactions', error);
      });
  }

  fetchTraceTransactions = (address) => {
    return Promise
      .all([
        this.context.api.trace
          .filter({
            fromBlock: 0,
            fromAddress: address
          }),
        this.context.api.trace
          .filter({
            fromBlock: 0,
            toAddress: address
          })
      ])
      .then(([fromTransactions, toTransactions]) => {
        const transactions = [].concat(fromTransactions, toTransactions);

        return transactions.map(transaction => ({
          from: transaction.action.from,
          to: transaction.action.to,
          blockNumber: transaction.blockNumber,
          hash: transaction.transactionHash
        }));
      });
  }
}

function mapStateToProps (state) {
  const { isTest, traceMode } = state.nodeStatus;

  return {
    isTest,
    traceMode
  };
}

function mapDispatchToProps (dispatch) {
  return bindActionCreators({}, dispatch);
}

export default connect(
  mapStateToProps,
  mapDispatchToProps
)(Transactions);
