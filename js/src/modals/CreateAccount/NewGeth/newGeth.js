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

import { SelectionList } from '~/ui';

import GethCard from '../GethCard';
import styles from '../createAccount.css';

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

    return gethAccountsAvailable.length
      ? (
        <div>
          <div className={ styles.summary }>
            <FormattedMessage
              id='createAccount.newGeth.available'
              defaultMessage='There are currently {count} importable keys available from the Geth keystore which are not already available on your Parity instance. Select the accounts you wish to import and move to the next step to complete the import.'
              values={ {
                count: gethAccountsAvailable.length
              } }
            />
          </div>
          { this.renderList(gethAccountsAvailable, gethAddresses) }
        </div>
      )
      : (
        <div className={ styles.summary }>
          <FormattedMessage
            id='createAccount.newGeth.noKeys'
            defaultMessage='There are currently no importable keys available from the Geth keystore, which are not already available on your Parity instance'
          />
        </div>
      );
  }

  renderList (gethAccountsAvailable) {
    return (
      <SelectionList
        isChecked={ this.isSelected }
        items={ gethAccountsAvailable }
        noStretch
        onSelectClick={ this.onSelect }
        renderItem={ this.renderAccount }
      />
    );
  }

  renderAccount = (account, index) => {
    return (
      <GethCard
        address={ account.address }
        balance={ account.balance }
        name={ `Geth Account ${index + 1}` }
      />
    );
  }

  isSelected = (account) => {
    const { gethAddresses } = this.props.store;

    return gethAddresses.includes(account.address);
  }

  onSelect = (account) => {
    const { store } = this.props;

    store.selectGethAccount(account.address);
  }
}
