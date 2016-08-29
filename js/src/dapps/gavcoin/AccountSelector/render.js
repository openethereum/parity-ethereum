import React from 'react';

import { MenuItem } from 'material-ui';

import styles from './style.css';

const { IdentityIcon } = window.parity.react;

export function renderAccounts (accounts, options = {}) {
  return accounts
    .filter((account) => options.all ? true : account.uuid)
    .map((account) => {
      const balance = options.gavBalance
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
