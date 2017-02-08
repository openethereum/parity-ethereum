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
import { Button, Container, Page, SectionList } from '~/ui';
import { AddIcon, LockedIcon, UnlockedIcon } from '~/ui/Icons';

import ConfirmClose from './ConfirmClose';
import ConfirmOpen from './ConfirmOpen';
import NameLayout from './NameLayout';
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
        buttons={ [
          <Button
            icon={ <AddIcon /> }
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
        <ConfirmClose store={ this.store } />
        <ConfirmOpen store={ this.store } />
        <VaultCreate store={ this.store } />
        <SectionList
          items={ vaults }
          renderItem={ this.renderItem }
        />
      </Page>
    );
  }

  renderItem = (item) => {
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
        <NameLayout { ...item } />
        {
          item.isOpen
            ? <UnlockedIcon className={ styles.iconLock } />
            : <LockedIcon className={ styles.iconLock } />
        }
      </Container>
    );
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
