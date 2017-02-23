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

import { SectionList } from '~/ui';
import GethCard from '../GethCard';

import styles from '../createAccount.css';

@observer
export default class AccountDetailsGeth extends Component {
  static propTypes = {
    store: PropTypes.object.isRequired
  }

  render () {
    const { gethAccountsAvailable, gethImported } = this.props.store;

    const accounts = gethAccountsAvailable.filter((account) => gethImported.includes(account.address));

    return (
      <div>
        <div className={ styles.summary }>
          <FormattedMessage
            id='createAccount.accountDetailsGeth.imported'
            defaultMessage='You have completed the import of {number} addresses from the Geth keystore. These will now be available in your accounts list as a normal account, along with their associated balances on the network.'
            values={ {
              number: gethImported.length
            } }
          />
        </div>
        <SectionList
          items={ accounts }
          noStretch
          renderItem={ this.renderAccount }
        />
      </div>
    );
  }

  renderAccount = (account, index) => {
    return (
      <GethCard
        address={ account.address }
        balance={ account.balance }
        name={ `Geth Import ${index + 1}` }
      />
    );
  }
}
