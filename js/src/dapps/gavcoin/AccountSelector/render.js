import React from 'react';

import { MenuItem } from 'material-ui';

import styles from './style.css';

const { IdentityIcon } = window.parity.react;

export function renderAccounts (accounts, gavBalance) {
  return accounts.map((account) => {
    const balance = gavBalance
      ? `${account.gavBalance}GAV`
      : `${account.ethBalance}ΞTH`;

    const item = (
      <div className={ styles.selectedaccount }>
        <div className={ styles.image }>
          <IdentityIcon inline center address={ account.address } />
        </div>
        <div className={ styles.details }>
          <div className={ styles.name }>{ account.name }</div>
          <div className={ styles.balance }>{ balance }</div>
        </div>
      </div>
    );

    return (
      <MenuItem
        className={ styles.selectaccount }
        key={ account.address }
        value={ account }
        label={ item }>
        { item }
      </MenuItem>
    );
  });
}
