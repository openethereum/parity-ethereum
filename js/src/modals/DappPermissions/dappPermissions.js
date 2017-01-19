// Copyright 2015, 2016 Parity Technologies (UK) Ltd.
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

import { Checkbox } from 'material-ui';
import { List, ListItem } from 'material-ui/List';
import { observer } from 'mobx-react';
import React, { Component, PropTypes } from 'react';
import { FormattedMessage } from 'react-intl';

import { Button, IdentityIcon, Modal } from '~/ui';
import { DoneIcon } from '~/ui/Icons';

import styles from './dappPermissions.css';

@observer
export default class DappPermissions extends Component {
  static propTypes = {
    store: PropTypes.object.isRequired
  };

  render () {
    const { store } = this.props;

    if (!store.modalOpen) {
      return null;
    }

    return (
      <Modal
        actions={ [
          <Button
            icon={ <DoneIcon /> }
            key='done'
            label={
              <FormattedMessage
                id='dapps.permissions.button.done'
                defaultMessage='Done'
              />
            }
            onClick={ store.closeModal }
          />
        ] }
        compact
        title={
          <FormattedMessage
            id='dapps.permissions.label'
            defaultMessage='visible dapp accounts'
          />
        }
        visible
      >
        <List>
          { this.renderListItems() }
        </List>
      </Modal>
    );
  }

  renderListItems () {
    const { store } = this.props;

    return store.accounts.map((account) => {
      const onCheck = () => {
        store.selectAccount(account.address);
      };

      // TODO: Once new modal & account selection is in, this should be updated
      // to conform to the new (as of this code WIP) look & feel for selection.
      // For now in the current/old style, not as pretty but consistent.
      return (
        <ListItem
          className={
            account.checked
              ? styles.selected
              : styles.unselected
          }
          key={ account.address }
          leftCheckbox={
            <Checkbox
              checked={ account.checked }
              onCheck={ onCheck }
            />
          }
          primaryText={
            <div className={ styles.item }>
              <IdentityIcon address={ account.address } />
              <div className={ styles.info }>
                <h3 className={ styles.name }>
                  { account.name }
                </h3>
                <div className={ styles.address }>
                  { account.address }
                </div>
                <div className={ styles.description }>
                  { account.description }
                </div>
              </div>
            </div>
          }
        />
      );
    });
  }
}
