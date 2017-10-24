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

import { VaultAccounts, VaultCreate, VaultLock, VaultMeta, VaultUnlock } from '~/modals';
import { Button, Container, Page, SectionList, VaultCard } from '~/ui';
import { AccountsIcon, AddIcon, EditIcon, LockedIcon, UnlockedIcon } from '~/ui/Icons';

import Store from './store';
import styles from './vaults.css';

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
        <VaultAccounts vaultStore={ this.vaultStore } />
        <VaultCreate vaultStore={ this.vaultStore } />
        <VaultLock vaultStore={ this.vaultStore } />
        <VaultMeta vaultStore={ this.vaultStore } />
        <VaultUnlock vaultStore={ this.vaultStore } />
        { this.renderList() }
      </Page>
    );
  }

  renderList () {
    const { vaults } = this.vaultStore;

    if (!vaults || !vaults.length) {
      return (
        <Container className={ styles.empty }>
          <FormattedMessage
            id='vaults.empty'
            defaultMessage='There are currently no vaults to display.'
          />
        </Container>
      );
    }

    return (
      <SectionList
        items={ vaults }
        renderItem={ this.renderVault }
      />
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
    const onClickEdit = () => {
      this.onOpenEdit(name);
      return false;
    };
    const onClickOpen = () => {
      isOpen
        ? this.onOpenLockVault(name)
        : this.onOpenUnlockVault(name);
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
                icon={ <EditIcon /> }
                key='edit'
                label={
                  <FormattedMessage
                    id='vaults.button.edit'
                    defaultMessage='edit'
                  />
                }
                onClick={ onClickEdit }
              />,
              <Button
                icon={ <LockedIcon /> }
                key='close'
                label={
                  <FormattedMessage
                    id='vaults.button.close'
                    defaultMessage='close'
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
                    defaultMessage='open'
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

  onOpenAccounts = (name) => {
    this.vaultStore.openAccountsModal(name);
  }

  onOpenCreate = () => {
    this.vaultStore.openCreateModal();
  }

  onOpenEdit = (name) => {
    this.vaultStore.openMetaModal(name);
  }

  onOpenLockVault = (name) => {
    this.vaultStore.openLockModal(name);
  }

  onOpenMeta = (name) => {
    this.vaultStore.openMetaModal(name);
  }

  onOpenUnlockVault = (name) => {
    this.vaultStore.openUnlockModal(name);
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
