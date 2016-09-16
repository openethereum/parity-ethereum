import React, { Component, PropTypes } from 'react';
import {List, ListItem, MakeSelectable} from 'material-ui/List';
import Subheader from 'material-ui/Subheader';
import Avatar from 'material-ui/Avatar';

import IdentityIcon from '../../IdentityIcon';

import styles from './account-selector.css';

export default class AccountSelector extends Component {

  static propTypes = {
    list: PropTypes.array,
    selected: PropTypes.object,
    handleSetSelected: PropTypes.func
  };

  state = {
    open: false
  };

  render () {
    let nestedAccounts = this.renderAccounts(this.props.list);
    let selectedAccount = this.renderAccount(
      this.props.selected,
      {
        nestedItems: nestedAccounts,
        open: this.state.open,
        onClick: this.onToggleOpen.bind(this),
        autoGenerateNestedIndicator: false
      }
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
      .map((account, index) => this.renderAccount(account, {key: index}));
  }

  renderAccount (account, props) {
    let icon = (<IdentityIcon
      inline center
      address={ account.address } />
    );

    let avatar = (<Avatar
      className={ styles.avatar }
      backgroundColor='none'
      icon={ icon } />
    );

    return (
      <ListItem
        onClick={ this.onSelectAccount.bind(this, account.address) }
        value={ account.address }
        primaryText={ account.name }
        secondaryText={ account.address }
        leftAvatar={avatar}
        { ...props } />
    );
  }

  onToggleOpen () {
    this.setState({ open: !this.state.open });
  }

  onSelectAccount (address) {
    this.props.handleSetSelected(address);
    this.onToggleOpen();
  }

}
