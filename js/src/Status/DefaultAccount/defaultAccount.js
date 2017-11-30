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

import React, { Component } from 'react';
import { observer } from 'mobx-react';

import List from 'semantic-ui-react/dist/commonjs/elements/List';
import Popup from 'semantic-ui-react/dist/commonjs/modules/Popup';
import IdentityIcon from '@parity/ui/lib/IdentityIcon';

import AccountStore from '../../ParityBar/accountStore';
import AccountItem from './AccountItem';
import styles from './defaultAccount.css';

@observer
class DefaultAccount extends Component {
  state = {
    isOpen: false
  }

  componentWillMount () {
    const { api } = this.context;

    this.accountStore = AccountStore.get(api);
  }

  handleOpen = () => {
    this.setState({ isOpen: true });
  }

  handleClose = () => {
    this.setState({ isOpen: false });
  }

  handleMakeDefault = (address) => {
    this.handleClose();
    if (address === this.accountStore.defaultAddress) { return; }
    this.accountStore.makeDefaultAccount(address);
  }

  render () {
    const { accounts, defaultAccount } = this.accountStore;

    if (!accounts || !defaultAccount) { return null; }

    return (
      <Popup
        wide='very'
        trigger={
          <IdentityIcon
            address={ defaultAccount } button
            center
            className={ styles.defaultAccount }
          />
        }
        content={
          <List divided relaxed='very' selection>
            {accounts
              .map(account => (
                <AccountItem
                  key={ account.address }
                  isDefault={ account.address === defaultAccount }
                  address={ account.address }
                  name={ account.name }
                  onClick={ this.handleMakeDefault }
                />
              ))}
          </List>
        }
        offset={ 13 } // Empirically looks better
        on='click'
        open={ this.state.isOpen }
        onClose={ this.handleClose }
        onOpen={ this.handleOpen }
        position='bottom right'
      />
    );
  }
}

export default DefaultAccount;
