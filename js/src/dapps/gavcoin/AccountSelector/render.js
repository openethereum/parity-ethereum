import React from 'react';

import { MenuItem } from 'material-ui';

const { IdentityIcon } = window.parity.react;

export function renderAccounts (accounts, gavBalance) {
  return accounts.map((account) => {
    const balance = gavBalance
      ? `${account.gavBalance}GAV`
      : `${account.ethBalance}ΞTH`;
    const identityIcon = (
      <IdentityIcon inline center address={ account.address } />
    );
    const icon = (
      <div className='iconimg'>
        { identityIcon }
      </div>
    );
    const label = (
      <div className='selectedaccount'>
        <div className='image'>
          { identityIcon }
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
        primaryText={ account.name }
        value={ account }
        label={ label }
        leftIcon={ icon } />
    );
  });
}
