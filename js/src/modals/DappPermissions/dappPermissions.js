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

import { AccountCard, Portal, SelectionList } from '~/ui';

@observer
export default class DappPermissions extends Component {
  static propTypes = {
    permissionStore: PropTypes.object.isRequired
  };

  render () {
    const { permissionStore } = this.props;

    if (!permissionStore.modalOpen) {
      return null;
    }

    return (
      <Portal
        onClose={ permissionStore.closeModal }
        open
        title={
          <FormattedMessage
            id='dapps.permissions.label'
            defaultMessage='visible dapp accounts'
          />
        }
      >
        <SelectionList
          items={ permissionStore.accounts }
          noStretch
          onDefaultClick={ this.onMakeDefault }
          onSelectClick={ this.onSelect }
          renderItem={ this.renderAccount }
        />
      </Portal>
    );
  }

  onMakeDefault = (account) => {
    this.props.permissionStore.setDefaultAccount(account.address);
  }

  onSelect = (account) => {
    this.props.permissionStore.selectAccount(account.address);
  }

  renderAccount = (account) => {
    return (
      <AccountCard
        account={ account }
      />
    );
  }
}
