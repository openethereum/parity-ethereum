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

import { Portal } from '~/ui';
import { AddCircleIcon } from '~/ui/Icons';

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
    if (!this.store.isOpen) {
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
        <div className={ styles.layout }>
          { this.renderAvailableList() }
          { this.renderOpenedList() }
        </div>
      </Portal>
    );
  }

  renderAvailableList () {
    const { listAll } = this.store;
    let items;

    if (!listAll || !listAll.length) {
      items = (
        <div className={ [styles.item, styles.empty].join(' ') }>
          <FormattedMessage
            id='vaults.listAll.empty'
            defaultMessage='No vaults have been created'
          />
        </div>
      );
    } else {
      items = listAll;
    }

    return (
      <div className={ styles.list }>
        <div className={ styles.item }>
          <AddCircleIcon />
        </div>
        { items }
      </div>
    );
  }

  renderOpenedList () {
    const { listOpened } = this.store;
    let items;

    if (!listOpened || !listOpened.length) {
      items = (
        <div className={ styles.item }>
          <div className={ styles.empty }>
            <FormattedMessage
              id='vaults.listOpened.empty'
              defaultMessage='No vaults have been opened'
            />
          </div>
        </div>
      );
    } else {
      items = listOpened;
    }

    return (
      <div className={ [styles.list, styles.reverse].join(' ') }>
        <div className={ styles.item }>
          <div className={ styles.content }>
            <AddCircleIcon />
          </div>
        </div>
        { items }
      </div>
    );
  }

  onClose = () => {
    this.store.closeModal();
  }
}
