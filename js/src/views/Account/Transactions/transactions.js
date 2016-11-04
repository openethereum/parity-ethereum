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

import { fetchAccountTransactions } from '../../../redux/providers/blockchainActions';
import { Container, ContainerTitle } from '../../../ui';

import Transaction from './Transaction';

import styles from './transactions.css';

class Transactions extends Component {
  static contextTypes = {
    api: PropTypes.object.isRequired
  }

  static propTypes = {
    address: PropTypes.string.isRequired,
    fetchAccountTransactions: PropTypes.func.isRequired,

    accountInfo: PropTypes.object,
    accounts: PropTypes.object,
    contacts: PropTypes.object,
    contracts: PropTypes.object,
    tokens: PropTypes.object,
    isTest: PropTypes.bool,
    traceMode: PropTypes.bool
  }

  componentWillMount () {
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
    const { accountInfo } = this.props;

    const isLoading = !accountInfo || accountInfo.loading;

    if (isLoading) {
      return (
        <LinearProgress mode='indeterminate' />
      );
    }

    const { transactions } = accountInfo;

    if (!transactions.length) {
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
    const { address, accounts, contacts, contracts, tokens, isTest, accountInfo } = this.props;
    const { transactions } = accountInfo;

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
            isTest={ isTest }
          />
        );
      });
  }

  getTransactions (props = this.props) {
    const { address, fetchAccountTransactions } = props;
    fetchAccountTransactions(address);
  }
}

function mapStateToProps (_, initProps) {
  const { address } = initProps;

  return (state) => {
    const { isTest, traceMode } = state.nodeStatus;
    const { accounts, contacts, contracts } = state.personal;
    const { tokens } = state.balances;

    const accountInfo = state.blockchain.accounts[address];

    return {
      isTest,
      traceMode,
      accounts,
      contacts,
      contracts,
      tokens,
      accountInfo
    };
  };
}

function mapDispatchToProps (dispatch) {
  return bindActionCreators({
    fetchAccountTransactions
  }, dispatch);
}

export default connect(
  mapStateToProps,
  mapDispatchToProps
)(Transactions);
