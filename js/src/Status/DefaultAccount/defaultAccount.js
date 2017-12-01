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
import PropTypes from 'prop-types';
import { observer } from 'mobx-react';

import List from 'semantic-ui-react/dist/commonjs/elements/List';
import Popup from 'semantic-ui-react/dist/commonjs/modules/Popup';
import IdentityIcon from '@parity/ui/lib/IdentityIcon';

import AccountItem from './AccountItem';
import styles from './defaultAccount.css';

@observer
class DefaultAccount extends Component {
  state = {
    isOpen: false
  }

  static propTypes = {
    accountStore: PropTypes.object.isRequired
  }

  handleOpen = () => {
    this.setState({ isOpen: true });
  }

  handleClose = () => {
    this.setState({ isOpen: false });
  }

  handleMakeDefault = (address) => {
    this.handleClose();
    if (address === this.props.accountStore.defaultAddress) { return; }
    this.props.accountStore.makeDefaultAccount(address);
  }

  render () {
    const { accounts, defaultAccount: defaultAddress } = this.props.accountStore;
    const defaultAccount = accounts.find(({ address }) => address === defaultAddress);

    if (!accounts || !defaultAccount) { return null; }

    return (
      <Popup
        wide='very'
        className={ styles.popup }
        trigger={
          <IdentityIcon
            address={ defaultAddress } button
            center
            className={ styles.defaultAccount }
          />
        }
        content={
          <div>
            <List relaxed='very' selection className={ [styles.list, styles.isDefault, accounts.length > 1 && styles.hasOtherAccounts].join(' ') }>
              <AccountItem
                isDefault
                account={ defaultAccount }
              />
            </List>
            {accounts.length > 1 &&
              <List relaxed='very' selection className={ styles.list } divided>
                {accounts
                  .filter(({ address }) => address !== defaultAddress)
                  .map(account => (
                    <AccountItem
                      key={ account.address }
                      account={ account }
                      onClick={ this.handleMakeDefault }
                    />
                  ))}
              </List>
            }
          </div>
        }
        offset={ 13 } // Empirically looks better
        on='click'
        hideOnScroll
        open={ this.state.isOpen }
        onClose={ this.handleClose }
        onOpen={ this.handleOpen }
        position='bottom right'
      />
    );
  }
}

export default DefaultAccount;
