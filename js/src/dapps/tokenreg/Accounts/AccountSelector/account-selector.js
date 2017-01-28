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

import React, { Component, PropTypes } from 'react';
import { List, ListItem } from 'material-ui/List';
import Subheader from 'material-ui/Subheader';
import Avatar from 'material-ui/Avatar';

import IdentityIcon from '../../IdentityIcon';

import styles from './account-selector.css';

class AccountSelectorItem extends Component {
  static propTypes = {
    onSelectAccount: PropTypes.func.isRequired,
    account: PropTypes.object.isRequired
  };

  render () {
    const account = this.props.account;

    const props = Object.assign({}, this.props);

    delete props.account;
    delete props.onSelectAccount;

    const icon = (
      <IdentityIcon
        inline center
        address={ account.address }
      />
    );

    const avatar = (
      <Avatar
        className={ styles.avatar }
        backgroundColor='none'
        icon={ icon }
      />
    );

    return (
      <ListItem
        onClick={ this.onSelectAccount }
        value={ account.address }
        primaryText={ account.name }
        secondaryText={ account.address }
        leftAvatar={ avatar }
        { ...props }
      />
    );
  }

  onSelectAccount = () => {
    this.props.onSelectAccount(this.props.account.address);
  }
}

export default class AccountSelector extends Component {
  static propTypes = {
    list: PropTypes.array.isRequired,
    selected: PropTypes.object.isRequired,
    handleSetSelected: PropTypes.func.isRequired,
    onAccountChange: PropTypes.func
  };

  state = {
    open: false
  };

  render () {
    const nestedAccounts = this.renderAccounts(this.props.list);
    const selectedAccount = (
      <AccountSelectorItem
        account={ this.props.selected }
        nestedItems={ nestedAccounts }
        open={ this.state.open }
        onSelectAccount={ this.onToggleOpen }
        autoGenerateNestedIndicator={ false }
        nestedListStyle={ { maxHeight: '14em', overflow: 'auto' } }
      />
    );

    return (
      <div className={ styles['account-selector'] }>
        <List>
          <Subheader>Select an account</Subheader>
          { selectedAccount }
        </List>
      </div>
    );
  }

  renderAccounts (accounts) {
    return accounts
      .map((account, index) => (
        <AccountSelectorItem
          account={ account }
          onSelectAccount={ this.onSelectAccount }
          key={ index }
        />
      ));
  }

  onToggleOpen = () => {
    this.setState({ open: !this.state.open });

    if (typeof this.props.onAccountChange === 'function') {
      this.props.onAccountChange();
    }
  }

  onSelectAccount = (address) => {
    this.props.handleSetSelected(address);
    this.onToggleOpen();
  }
}
