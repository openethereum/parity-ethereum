import React, { Component } from 'react';

import Account from '../../../Account';
import styles from './AccountPage.css';

import accountPageData from './AccountPage.data';

export default class AccountPage extends Component {

  render () {
    return (
      <div>
        <h1>Account</h1>
        { this.renderAccounts() }
      </div>
    );
  }

  renderAccounts () {
    return accountPageData.map(acc => {
      return (
        <div className={ styles.accountContainer } key={ acc.address }>
          <Account { ...acc } className={ styles.account } />
          { this.renderAccountInfo(acc) }
        </div>
      );
    });
  }

  renderAccountInfo (acc) {
    return (
      <div className={ styles.accountInfo }>
        <div>Chain: { acc.chain }</div>
        <div>Address: { acc.address }</div>
        <div>Balance: { acc.balance.div(1e18).toFormat(3) } ETH</div>
        <div>Name: { acc.name || 'empty' }</div>
      </div>
    );
  }

}
