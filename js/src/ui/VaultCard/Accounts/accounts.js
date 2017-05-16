// Copyright 2015-2017 Parity Technologies (UK) Ltd.
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

import React, { PropTypes } from 'react';
import { FormattedMessage } from 'react-intl';

import DappLink from '~/ui/DappLink';
import IdentityIcon from '~/ui/IdentityIcon';

import styles from '../vaultCard.css';

export default function Accounts ({ accounts, hideAccounts }) {
  if (hideAccounts) {
    return null;
  }

  if (!accounts || !accounts.length) {
    return (
      <div className={ styles.empty }>
        <FormattedMessage
          id='vaults.accounts.empty'
          defaultMessage='There are no accounts in this vault'
        />
      </div>
    );
  }

  return (
    <div className={ styles.accounts }>
      {
        accounts.map((address) => (
          <DappLink
            key={ address }
            to={ `/accounts/${address}` }
          >
            <IdentityIcon
              address={ address }
              center
              className={ styles.account }
            />
          </DappLink>
        ))
      }
    </div>
  );
}

Accounts.propTypes = {
  accounts: PropTypes.array,
  hideAccounts: PropTypes.bool
};
