import React, { Component, PropTypes } from 'react';

import styles from './accountItem.css';

const { IdentityIcon } = window.parity.react;

export default class AccountItem extends Component {
  static propTypes = {
    account: PropTypes.object,
    gavBalance: PropTypes.bool
  };

  render () {
    const { account, gavBalance } = this.props;

    let balance;
    let token;

    if (gavBalance) {
      if (account.gavBalance) {
        balance = account.gavBalance;
        token = 'GAV';
      }
    } else {
      if (account.ethBalance) {
        balance = account.ethBalance;
        token = 'ΞTH';
      }
    }

    return (
      <div className={ styles.account }>
        <div className={ styles.image }>
          <IdentityIcon
            inline center
            address={ account.address } />
        </div>
        <div className={ styles.details }>
          <div className={ styles.name }>
            { account.name || account.address }
          </div>
          <div className={ styles.balance }>
            { balance }<small> { token }</small>
          </div>
        </div>
      </div>
    );
  }
}
