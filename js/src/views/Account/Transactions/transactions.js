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

import React, { Component, PropTypes } from 'react';
import { connect } from 'react-redux';
import { bindActionCreators } from 'redux';
import LinearProgress from 'material-ui/LinearProgress';

import etherscan from '../../../3rdparty/etherscan';
import { Container, ContainerTitle } from '../../../ui';

import Transaction from './Transaction';

import styles from './transactions.css';

class Transactions extends Component {
  static contextTypes = {
    api: PropTypes.object.isRequired
  }

  static propTypes = {
    address: PropTypes.string.isRequired,
    accounts: PropTypes.object,
    contacts: PropTypes.object,
    contracts: PropTypes.object,
    tokens: PropTypes.object,
    isTest: PropTypes.bool,
    traceMode: PropTypes.bool
  }

  state = {
    transactions: [],
    loading: true,
    callInfo: {}
  }

  componentDidMount () {
    if (this.props.traceMode !== undefined) {
      this.getTransactions(this.props);
    }
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
      <Container>
        <ContainerTitle title='transactions' />
        { this.renderTransactions() }
      </Container>
    );
  }

  renderTransactions () {
    const { loading, transactions } = this.state;

    if (loading) {
      return (
        <LinearProgress mode='indeterminate' />
      );
    } else if (!transactions.length) {
      return (
        <div className={ styles.infonone }>
          No transactions were found for this account
        </div>
      );
    }

    return (
      <div className={ styles.transactions }>
        <table>
          <tbody>
            { this.renderRows() }
          </tbody>
        </table>
        { this.renderEtherscanFooter() }
      </div>
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

  renderRows () {
    const { address, accounts, contacts, contracts, tokens, isTest } = this.props;
    const { transactions } = this.state;

    return (transactions || [])
      .sort((tA, tB) => {
        return tB.blockNumber.comparedTo(tA.blockNumber);
      })
      .slice(0, 25)
      .map((transaction, index) => {
        return (
          <Transaction
            key={ index }
            transaction={ transaction }
            address={ address }
            accounts={ accounts }
            contacts={ contacts }
            contracts={ contracts }
            tokens={ tokens }
            isTest={ isTest } />
        );
      });
  }

  getTransactions = (props) => {
    const { isTest, address, traceMode } = props;

    return this
      .fetchTransactions(isTest, address, traceMode)
      .then(transactions => {
        this.setState({
          transactions,
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
  const { accounts, contacts, contracts } = state.personal;
  const { tokens } = state.balances;

  return {
    isTest,
    traceMode,
    accounts,
    contacts,
    contracts,
    tokens
  };
}

function mapDispatchToProps (dispatch) {
  return bindActionCreators({}, dispatch);
}

export default connect(
  mapStateToProps,
  mapDispatchToProps
)(Transactions);
