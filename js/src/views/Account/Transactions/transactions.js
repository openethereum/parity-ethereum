import BigNumber from 'bignumber.js';
import React, { Component, PropTypes } from 'react';
import moment from 'moment';

import Api from '../../../api';
import etherscan from '../../../3rdparty/etherscan';
import Container from '../../../ui/Container';

import styles from '../style.css';

function formatHash (hash) {
  if (!hash || hash.length <= 25) {
    return hash;
  }

  return `${hash.substr(2, 11)}...${hash.slice(-11)}`;
}

function formatNumber (number) {
  return new BigNumber(number).toFormat();
}

function formatTime (time) {
  return moment(parseInt(time, 10) * 1000).fromNow(true);
}

function formatEther (value) {
  const ether = Api.format.fromWei(value);

  if (ether.gt(0)) {
    return `${ether.toFormat(8)} ΞTH`;
  }

  return null;
}

export default class Transactions extends Component {
  static propTypes = {
    address: PropTypes.string.isRequired
  }

  state = {
    transactions: []
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

  renderTransactions () {
    let transactions = null;

    if (this.state.transactions && this.state.transactions.length) {
      transactions = (this.state.transactions || []).map((tx) => {
        const hashLink = `https://etherscan.io/tx/${tx.hash}`;
        const fromLink = `https://etherscan.io/address/${tx.from}`;
        const toLink = `https://etherscan.io/address/${tx.to}`;

        return (
          <tr key={ tx.hash }>
            <td className={ styles.center }></td>
            <td className={ styles.center }>
              <a href={ fromLink } target='_blank'>{ formatHash(tx.from) }</a>
            </td>
            <td className={ styles.center }>
              <a href={ toLink } target='_blank'>{ formatHash(tx.to) }</a>
            </td>
            <td className={ styles.center }>
              <a href={ hashLink } target='_blank'>{ formatHash(tx.hash) }</a>
            </td>
            <td className={ styles.right }>
              { formatNumber(tx.blockNumber) }
            </td>
            <td className={ styles.right }>
              { formatTime(tx.timeStamp) }
            </td>
            <td className={ styles.right }>
              { formatEther(tx.value) }
            </td>
          </tr>
        );
      });
    }

    return (
      <table className={ styles.transactions }>
        <thead>
          <tr>
            <th>&nbsp;</th>
            <th>from</th>
            <th>to</th>
            <th>txhash</th>
            <th className={ styles.right }>block</th>
            <th className={ styles.right }>age</th>
            <th className={ styles.right }>value</th>
          </tr>
        </thead>
        <tbody>
          { transactions }
        </tbody>
      </table>
    );
  }

  getTransactions = () => {
    etherscan.account
      .transactions(this.props.address)
      .then((transactions) => {
        this.setState({
          transactions: transactions
        });
      });
  }
}
