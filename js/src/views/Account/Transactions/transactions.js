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
import { connect } from 'react-redux';
import { bindActionCreators } from 'redux';
import moment from 'moment';
import LinearProgress from 'material-ui/LinearProgress';

import format from '../../../api/format';
import etherscan from '../../../3rdparty/etherscan';
import { Container, IdentityIcon } from '../../../ui';

import styles from './transactions.css';

function formatHash (hash) {
  if (!hash || hash.length <= 21) {
    return hash;
  }

  return `${hash.substr(2, 9)}...${hash.slice(-9)}`;
}

function formatNumber (number) {
  return new BigNumber(number).toFormat();
}

function formatTime (time) {
  return moment(parseInt(time, 10) * 1000).fromNow(true);
}

function formatEther (value) {
  const ether = format.fromWei(value);

  if (ether.gt(0)) {
    return `${ether.toFormat(5)}`;
  }

  return null;
}

class Transactions extends Component {
  static propTypes = {
    address: PropTypes.string.isRequired,
    accounts: PropTypes.object,
    contacts: PropTypes.object,
    tokens: PropTypes.object,
    isTest: PropTypes.bool
  }

  state = {
    transactions: [],
    loading: true
  }

  componentDidMount () {
    this.getTransactions();
  }

  render () {
    return (
      <Container>
        { this.renderTransactions() }
      </Container>
    );
  }

  renderAddress (prefix, address) {
    const { accounts, contacts, tokens } = this.props;
    const account = (accounts || {})[address] || (contacts || {})[address] || (tokens || {})[address];
    const link = `${prefix}address/${address}`;
    const name = account
      ? account.name.toUpperCase()
      : formatHash(address);

    return (
      <td className={ styles.left }>
        <IdentityIcon
          inline center
          tokens={ tokens }
          address={ address } />
        <a
          href={ link }
          target='_blank'
          className={ styles.link }>
          { name }
        </a>
      </td>
    );
  }

  renderTransactions () {
    const { isTest } = this.props;
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

    const prefix = `https://${isTest ? 'testnet.' : ''}etherscan.io/`;
    const rows = (transactions || []).map((tx) => {
      const hashLink = `${prefix}tx/${tx.hash}`;
      const value = formatEther(tx.value);
      const token = value ? 'ΞTH' : null;
      const tosection = (tx.to && tx.to.length)
        ? this.renderAddress(prefix, tx.to)
        : (<td className={ `${styles.center}` }></td>);

      return (
        <tr key={ tx.hash }>
          <td className={ styles.center }></td>
          { this.renderAddress(prefix, tx.from) }
          { tosection }
          <td className={ styles.center }>
            <a href={ hashLink } target='_blank' className={ styles.link }>
              { formatHash(tx.hash) }
            </a>
          </td>
          <td className={ styles.right }>
            { formatNumber(tx.blockNumber) }
          </td>
          <td className={ styles.right }>
            { formatTime(tx.timeStamp) }
          </td>
          <td className={ styles.value }>
            { formatEther(tx.value) }<small> { token }</small>
          </td>
        </tr>
      );
    });

    return (
      <div className={ styles.transactions }>
        <table>
          <thead>
            <tr className={ styles.info }>
              <th>&nbsp;</th>
              <th className={ styles.left }>from</th>
              <th className={ styles.left }>to</th>
              <th className={ styles.center }>transaction</th>
              <th className={ styles.right }>block</th>
              <th className={ styles.right }>age</th>
              <th className={ styles.right }>value</th>
            </tr>
          </thead>
          <tbody>
            { rows }
          </tbody>
        </table>
        <div className={ styles.etherscan }>
          Transaction list powered by <a href='https://etherscan.io/' target='_blank'>etherscan.io</a>
        </div>
      </div>
    );
  }

  getTransactions = () => {
    const { isTest, address } = this.props;

    return etherscan.account
      .transactions(address, 0, isTest)
      .then((transactions) => {
        this.setState({
          transactions,
          loading: false
        });
      })
      .catch((error) => {
        console.error('getTransactions', error);
      });
  }
}

function mapStateToProps (state) {
  const { isTest } = state.nodeStatus;
  const { accounts, contacts } = state.personal;
  const { tokens } = state.balances;

  return {
    isTest,
    accounts,
    contacts,
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
