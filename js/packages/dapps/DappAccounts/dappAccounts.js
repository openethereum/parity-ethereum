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
import React, { Component } from 'react';
import PropTypes from 'prop-types';
import { FormattedMessage } from 'react-intl';

import { AccountCard, Page, SelectionList } from '@parity/ui';

import Store from './store';

@observer
export default class DappAccounts extends Component {
  static contextTypes = {
    api: PropTypes.object.isRequired
  };

  store = new Store(this.context.api);

  render () {
    return (
      <Page
        title={
          <FormattedMessage
            id='dapps.accounts.label'
            defaultMessage='visible dapp accounts'
          />
        }
      >
        <SelectionList
          items={ this.store.accounts }
          noStretch
          onDefaultClick={ this.onMakeDefault }
          onSelectClick={ this.onSelect }
          renderItem={ this.renderAccount }
        />
      </Page>
    );
  }

  onMakeDefault = (account) => {
    this.store.setDefaultAccount(account.address);
  }

  onSelect = (account) => {
    this.store.selectAccount(account.address);
  }

  renderAccount = (account) => {
    return (
      <AccountCard
        account={ account }
      />
    );
  }
}
