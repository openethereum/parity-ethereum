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
import { connect } from 'react-redux';

import { VaultAccounts, VaultClose, VaultCreate, VaultOpen } from '~/modals';
import { Button, Page, SectionList, VaultCard } from '~/ui';
import { AccountsIcon, AddIcon, LockedIcon, UnlockedIcon } from '~/ui/Icons';

import Store from './store';

@observer
class Vaults extends Component {
  static contextTypes = {
    api: PropTypes.object.isRequired
  };

  static propTypes = {
    accounts: PropTypes.object.isRequired
  };

  static Store = Store;

  vaultStore = Store.get(this.context.api);

  componentWillMount () {
    return this.vaultStore.loadVaults();
  }

  render () {
    const { vaults } = this.vaultStore;

    return (
      <Page
        buttons={ [
          <Button
            icon={ <AddIcon /> }
            key='create'
            label={
              <FormattedMessage
                id='vaults.button.add'
                defaultMessage='create vault'
              />
            }
            onClick={ this.onOpenCreate }
          />
        ] }
        title={
          <FormattedMessage
            id='vaults.title'
            defaultMessage='Vault Management'
          />
        }
      >
        <VaultClose vaultStore={ this.vaultStore } />
        <VaultOpen vaultStore={ this.vaultStore } />
        <VaultAccounts vaultStore={ this.vaultStore } />
        <VaultCreate vaultStore={ this.vaultStore } />
        <SectionList
          items={ vaults }
          renderItem={ this.renderVault }
        />
      </Page>
    );
  }

  renderVault = (vault) => {
    const { accounts } = this.props;
    const { isOpen, name } = vault;
    const vaultAccounts = Object
      .keys(accounts)
      .filter((address) => accounts[address].uuid && accounts[address].meta.vault === vault.name);

    const onClickAccounts = () => {
      this.onOpenAccounts(name);
      return false;
    };
    const onClickOpen = () => {
      isOpen
        ? this.onCloseVault(name)
        : this.onOpenVault(name);
      return false;
    };

    return (
      <VaultCard
        accounts={ vaultAccounts }
        buttons={
          isOpen
            ? [
              <Button
                icon={ <AccountsIcon /> }
                key='accounts'
                label={
                  <FormattedMessage
                    id='vaults.button.accounts'
                    defaultMessage='accounts'
                  />
                }
                onClick={ onClickAccounts }
              />,
              <Button
                icon={ <LockedIcon /> }
                key='close'
                label={
                  <FormattedMessage
                    id='vaults.button.close'
                    defaultMessage='close vault'
                  />
                }
                onClick={ onClickOpen }
              />
            ]
            : [
              <Button
                icon={ <UnlockedIcon /> }
                key='open'
                label={
                  <FormattedMessage
                    id='vaults.button.open'
                    defaultMessage='open vault'
                  />
                }
                onClick={ onClickOpen }
              />
            ]
        }
        vault={ vault }
      />
    );
  }

  onCloseVault = (name) => {
    this.vaultStore.openCloseModal(name);
  }

  onOpenAccounts = (name) => {
    this.vaultStore.openAccountsModal(name);
  }

  onOpenCreate = () => {
    this.vaultStore.openCreateModal();
  }

  onOpenVault = (name) => {
    this.vaultStore.openOpenModal(name);
  }
}

function mapStateToProps (state) {
  const { accounts } = state.personal;

  return { accounts };
}

export default connect(
  mapStateToProps,
  null
)(Vaults);
