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

import { VaultAccounts, VaultCreate } from '~/modals';
import { Button, Container, IdentityIcon, Page, SectionList } from '~/ui';
import { AccountsIcon, AddIcon, LockedIcon, UnlockedIcon } from '~/ui/Icons';

import ConfirmClose from './ConfirmClose';
import ConfirmOpen from './ConfirmOpen';
import NameLayout from './NameLayout';
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
        <ConfirmClose vaultStore={ this.vaultStore } />
        <ConfirmOpen vaultStore={ this.vaultStore } />
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
    const onClickAccounts = () => {
      this.onOpenAccounts(vault.name);
      return false;
    };
    const onClickOpen = () => {
      vault.isOpen
        ? this.onCloseVault(vault.name)
        : this.onOpenVault(vault.name);
      return false;
    };

    return (
      <Container
        className={ styles.container }
        hover={
          vault.isOpen
            ? this.renderVaultAccounts(vault)
            : null
        }
      >
        <NameLayout { ...vault } />
        {
          vault.isOpen
            ? <UnlockedIcon className={ styles.statusIcon } />
            : <LockedIcon className={ styles.statusIcon } />
        }
        {
          vault.isOpen
            ? (
              <div className={ styles.buttonRow }>
                <Button
                  icon={ <AccountsIcon /> }
                  label={
                    <FormattedMessage
                      id='vaults.button.accounts'
                      defaultMessage='accounts'
                    />
                  }
                  onClick={ onClickAccounts }
                />
                <Button
                  icon={ <LockedIcon /> }
                  label={
                    <FormattedMessage
                      id='vaults.button.close'
                      defaultMessage='close vault'
                    />
                  }
                  onClick={ onClickOpen }
                />
              </div>
            )
            : (
              <div className={ styles.buttonRow }>
                <Button
                  icon={ <UnlockedIcon /> }
                  label={
                    <FormattedMessage
                      id='vaults.button.open'
                      defaultMessage='open vault'
                    />
                  }
                  onClick={ onClickOpen }
                />
              </div>
            )
        }
      </Container>
    );
  }

  renderVaultAccounts = (vault) => {
    const { accounts } = this.props;
    const vaultAccounts = Object
      .keys(accounts)
      .filter((address) => accounts[address].uuid && accounts[address].meta.vault === vault.name);

    if (!vaultAccounts.length) {
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
          vaultAccounts.map((address) => {
            return (
              <IdentityIcon
                address={ address }
                className={ styles.account }
                key={ address }
              />
            );
          })
        }
      </div>
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
