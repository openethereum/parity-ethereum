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

import { snakeCase } from 'lodash';
import { observer } from 'mobx-react';
import React, { Component, PropTypes } from 'react';
import { FormattedMessage } from 'react-intl';

import { Portal } from '~/ui';
import { AddCircleIcon, LockedIcon, NextIcon } from '~/ui/Icons';

import Create from './Create';
import Store from './store';
import styles from './vaults.css';

@observer
export default class Vaults extends Component {
  static contextTypes = {
    api: PropTypes.object.isRequired
  };

  static Store = Store;

  store = Store.get(this.context.api);

  render () {
    const { isOpen, listAll, listOpened } = this.store;

    if (!isOpen) {
      return null;
    }

    return (
      <Portal
        onClose={ this.onClose }
        open
        title={
          <FormattedMessage
            id='vaults.title'
            defaultMessage='Vault Management'
          />
        }
      >
        <div className={ styles.body }>
          <Create store={ this.store } />
          {
            this.renderList(listAll, true, (
              <FormattedMessage
                id='vaults.listAll.empty'
                defaultMessage='No vaults have been created'
              />
            ))
          }
          {
            this.renderList(listOpened, false, (
              <FormattedMessage
                id='vaults.listOpened.empty'
                defaultMessage='No vaults have been opened'
              />
            ))
          }
        </div>
      </Portal>
    );
  }

  renderList (list, allList, emptyMessage) {
    return (
      <div className={ styles.list }>
        {
          allList
            ? (
              <div className={ styles.item }>
                <div
                  className={ styles.content }
                  onClick={ this.onOpenAdd }
                >
                  <AddCircleIcon className={ styles.iconAdd } />
                </div>
              </div>
            )
            : null
        }
        { this.renderListItems(list, allList, emptyMessage) }
      </div>
    );
  }

  renderListItems (list, allList, emptyMessage) {
    if (!list || !list.length) {
      return (
        <div className={ styles.item }>
          <div className={ styles.empty }>
            { emptyMessage }
          </div>
        </div>
      );
    }

    return list.map((name, index) => {
      const onClick = () => {
        return allList
          ? this.onOpenVault(name)
          : this.onCloseVault(name);
      };

      return (
        <div
          className={ styles.item }
          key={ `${snakeCase(name)}_${index}` }
          onClick={ onClick }
        >
          <div className={ styles.content }>
            <div className={ styles.name }>
              { name }
            </div>
            {
              allList
                ? <NextIcon className={ styles.iconMove } />
                : <LockedIcon className={ styles.iconMove } />
            }
          </div>
        </div>
      );
    });
  }

  onClose = () => {
    this.store.closeModal();
  }

  onCloseVault = (name) => {
    console.log(`closing vault ${name}`);
  }

  onOpenAdd = () => {
    this.store.openAdd();
  }

  onOpenVault = (name) => {
    console.log(`opening vault ${name}`);
  }
}
