import React from 'react';

import { MenuItem } from 'material-ui';

const { IdentityIcon } = window.parity.react;

export function renderAccounts (accounts, gavBalance) {
  return accounts.map((account) => {
    const balance = gavBalance
      ? `${account.gavBalance}GAV`
      : `${account.ethBalance}ΞTH`;

    const item = (
      <div className='selectedaccount'>
        <div className='image'>
          <IdentityIcon inline center address={ account.address } />
        </div>
        <div className='details'>
          <div className='name'>{ account.name }</div>
          <div className='balance'>{ balance }</div>
        </div>
      </div>
    );

    return (
      <MenuItem
        className='selectaccount'
        key={ account.address }
        value={ account }
        label={ item }>
        { item }
      </MenuItem>
    );
  });
}
