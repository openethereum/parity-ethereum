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

import { VaultCreate } from '~/modals';
import { ConfirmDialog, Container, IdentityIcon, Page, SectionList } from '~/ui';
import { AddCircleIcon, LockedIcon, UnlockedIcon } from '~/ui/Icons';

import Store from './store';
import styles from './vaults.css';

@observer
export default class Vaults extends Component {
  static contextTypes = {
    api: PropTypes.object.isRequired
  };

  static Store = Store;

  store = Store.get(this.context.api);

  componentWillMount () {
    return this.store.loadVaults();
  }

  render () {
    const { vaults } = this.store;

    return (
      <Page
        title={
          <FormattedMessage
            id='vaults.title'
            defaultMessage='Vault Management'
          />
        }
      >
        <VaultCreate store={ this.store } />

        { this.renderConfirmClose() }
        { this.renderConfirmOpen() }

        <SectionList
          items={ [{ isAddButton: true }].concat(vaults.peek()) }
          renderItem={ this.renderItem }
        />
      </Page>
    );
  }

  renderItem = (item) => {
    if (item.isAddButton) {
      return (
        <Container
          className={ styles.container }
          onClick={ this.onOpenCreate }
        >
          <AddCircleIcon className={ styles.iconAdd } />
          <div className={ styles.name }>
            <FormattedMessage
              id='vaults.button.add'
              defaultMessage='add vault'
            />
          </div>
        </Container>
      );
    }

    const onClick = () => {
      return item.isOpen
        ? this.onCloseVault(item.name)
        : this.onOpenVault(item.name);
    };

    return (
      <Container
        className={ styles.container }
        onClick={ onClick }
      >
        <IdentityIcon
          address={ item.name }
          center
          className={
            [
              styles.identityIcon,
              item.isOpen
                ? styles.inlocked
                : styles.locked
            ].join(' ')
          }
        />
        <div className={ styles.name }>
          { item.name }
        </div>
        {
          item.isOpen
            ? <UnlockedIcon className={ styles.iconMove } />
            : <LockedIcon className={ styles.iconMove } />
        }
      </Container>
    );
  }

  renderConfirmClose () {
    return (
      <ConfirmDialog
        onConfirm={ this.executeCloseVault }
        onDeny={ this.onDenyCloseVault }
        open={ this.store.isModalCloseOpen }
        title={
          <FormattedMessage
            id='vaults.closeConfirm.title'
            defaultMessage='Vault Close'
          />
        }
      />
    );
  }

  renderConfirmOpen () {
    return (
      <ConfirmDialog
        onConfirm={ this.executeOpenVault }
        onDeny={ this.onDenyOpenVault }
        open={ this.store.isModalOpenOpen }
        title={
          <FormattedMessage
            id='vaults.openConfirm.title'
            defaultMessage='Vault Open'
          />
        }
      />
    );
  }

  exceuteCloseVault = () => {
    return this.store.closeVault();
  }

  exceuteOpenVault = () => {
    return this.store.openVault();
  }

  onDenyCloseVault = () => {
    this.store.closeCloseModal();
  }

  onDenyOpenVault = () => {
    this.store.closeOpenModal();
  }

  onCloseVault = (name) => {
    this.store.openCloseModal(name);
  }

  onOpenCreate = () => {
    this.store.openCreateModal();
  }

  onOpenVault = (name) => {
    this.store.openOpenModal(name);
  }
}
