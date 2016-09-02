import BigNumber from 'bignumber.js';
import React, { Component, PropTypes } from 'react';
import moment from 'moment';
import LinearProgress from 'material-ui/LinearProgress';

import format from '../../../api/format';
import etherscan from '../../../3rdparty/etherscan';
import { Container, IdentityIcon } from '../../../ui';

import styles from '../style.css';

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

export default class Transactions extends Component {
  static contextTypes = {
    api: PropTypes.object,
    accounts: PropTypes.array,
    contacts: PropTypes.array,
    contracts: PropTypes.array,
    tokens: PropTypes.array
  }

  static propTypes = {
    address: PropTypes.string.isRequired
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
    const cmp = (_account) => _account.address === address;

    let account = this.context.accounts.find(cmp);
    if (!account) {
      account = this.context.contacts.find(cmp);
      if (!account) {
        account = this.context.contracts.find(cmp);
        if (!account) {
          account = this.context.tokens.find(cmp);
        }
      }
    }

    const link = `${prefix}address/${address}`;
    const name = account
      ? account.name.toUpperCase()
      : formatHash(address);

    return (
      <td className={ styles.left }>
        <IdentityIcon
          inline center
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
    let transactions = null;
    const prefix = this.state.isTest
      ? 'https://testnet.etherscan.io/'
      : 'https://etherscan.io/';

    if (this.state.transactions && this.state.transactions.length) {
      transactions = (this.state.transactions || []).map((tx) => {
        const hashLink = `${prefix}tx/${tx.hash}`;

        const tosection = (tx.to && tx.to.length)
          ? this.renderAddress(prefix, tx.to)
          : (<td className={ `${styles.center}` }></td>);

        return (
          <tr key={ tx.hash }>
            <td className={ styles.center }></td>
            { this.renderAddress(prefix, tx.from) }
            { tosection }
            <td className={ styles.center }>
              <a href={ hashLink } target='_blank' className={ styles.link }>{ formatHash(tx.hash) }</a>
            </td>
            <td className={ styles.right }>
              { formatNumber(tx.blockNumber) }
            </td>
            <td className={ styles.right }>
              { formatTime(tx.timeStamp) }
            </td>
            <td className={ styles.value }>
              { formatEther(tx.value) }<small> ΞTH</small>
            </td>
          </tr>
        );
      });

      return (
        <table className={ styles.transactions }>
          <thead>
            <tr className={ styles.info }>
              <th>&nbsp;</th>
              <th className={ styles.left }>from</th>
              <th className={ styles.left }>to</th>
              <th className={ styles.center }>txhash</th>
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
    } else if (this.state.loading) {
      return (
        <LinearProgress mode='indeterminate' />
      );
    }

    return (
      <div className={ styles.infonone }>
        No transactions were found for this account
      </div>
    );
  }

  getTransactions = () => {
    this.context.api.ethcore
      .netChain()
      .then((netChain) => {
        const isTest = netChain === 'morden';

        return etherscan.account
          .transactions(this.props.address, 0, isTest)
          .then((transactions) => {
            this.setState({
              transactions,
              isTest,
              loading: false
            });
          });
      });
  }
}
