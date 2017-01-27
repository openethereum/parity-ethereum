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

import { observer } from 'mobx-react';
import React, { Component, PropTypes } from 'react';
import { FormattedMessage } from 'react-intl';
import { Checkbox } from 'material-ui';

import { IdentityIcon } from '~/ui';

import styles from './newGeth.css';

@observer
export default class NewGeth extends Component {
  static contextTypes = {
    api: PropTypes.object.isRequired
  }

  static propTypes = {
    store: PropTypes.object.isRequired
  }

  render () {
    const { gethAccountsAvailable, gethAddresses } = this.props.store;

    if (!gethAccountsAvailable.length) {
      return (
        <div className={ styles.list }>
          <FormattedMessage
            id='createAccount.newGeth.noKeys'
            defaultMessage='There are currently no importable keys available from the Geth keystore, which are not already available on your Parity instance'
          />
        </div>
      );
    }

    const checkboxes = gethAccountsAvailable.map((account) => {
      const onSelect = (event) => this.onSelectAddress(event, account.address);

      const label = (
        <div className={ styles.selection }>
          <div className={ styles.icon }>
            <IdentityIcon
              address={ account.address }
              center
              inline
            />
          </div>
          <div className={ styles.detail }>
            <div className={ styles.address }>
              { account.address }
            </div>
            <div className={ styles.balance }>
              { account.balance } ETH
            </div>
          </div>
        </div>
      );

      return (
        <Checkbox
          checked={ gethAddresses.includes(account.address) }
          key={ account.address }
          label={ label }
          onCheck={ onSelect }
        />
      );
    });

    return (
      <div className={ styles.list }>
        { checkboxes }
      </div>
    );
  }

  onSelectAddress = (event, address) => {
    const { store } = this.props;

    store.selectGethAccount(address);
  }
}
