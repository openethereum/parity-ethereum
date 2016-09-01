import React, { Component } from 'react';

import AccountLink from '../../../AccountLink';
import styles from './AccountLinkPage.css';

import AccountLinkPageData from './AccountLinkPage.data';

export default class AccountLinkPage extends Component {

  render () {
    return (
      <div>
        <h1>Account Link</h1>
        { this.renderAccountsLinks() }
      </div>
    );
  }

  renderAccountsLinks () {
    return AccountLinkPageData.map(acc => {
      return (
        <div className={ styles.AccountLinksContainer } key={ acc.address }>
          <AccountLink { ...acc } className={ styles.link }>
            { acc.address }
          </AccountLink>
          { this.renderAccountLinkInfo(acc) }
        </div>
      );
    });
  }

  renderAccountLinkInfo (acc) {
    return (
      <div className={ styles.AccountLinksInfo }>
        <div>Chain: { acc.chain }</div>
        <div>Address: { acc.address }</div>
      </div>
    );
  }

}
