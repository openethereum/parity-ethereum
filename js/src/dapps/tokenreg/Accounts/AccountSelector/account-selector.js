import React, { Component, PropTypes } from 'react';
import { List, ListItem } from 'material-ui/List';
import Subheader from 'material-ui/Subheader';
import Avatar from 'material-ui/Avatar';

import IdentityIcon from '../../IdentityIcon';

import styles from './account-selector.css';

class AccountSelectorItem extends Component {

  static propTypes = {
    onSelectAccount: PropTypes.func,
    account: PropTypes.object
  };

  constructor () {
    super();

    this.onSelectAccount = this.onSelectAccount.bind(this);
  }

  render () {
    let account = this.props.account;

    let props = Object.assign({}, this.props);
    delete props.account;
    delete props.onSelectAccount;

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
        onClick={ this.onSelectAccount }
        value={ account.address }
        primaryText={ account.name }
        secondaryText={ account.address }
        leftAvatar={ avatar }
        { ...props } />
    );
  }

  onSelectAccount () {
    this.props.onSelectAccount(this.props.account.address);
  }

}

export default class AccountSelector extends Component {

  static propTypes = {
    list: PropTypes.array,
    selected: PropTypes.object,
    handleSetSelected: PropTypes.func
  };

  state = {
    open: false
  };

  constructor () {
    super();

    this.onToggleOpen = this.onToggleOpen.bind(this);
    this.onSelectAccount = this.onSelectAccount.bind(this);
  }

  render () {
    let nestedAccounts = this.renderAccounts(this.props.list);
    let selectedAccount = (
      <AccountSelectorItem
        account={ this.props.selected }
        nestedItems={ nestedAccounts }
        open={ this.state.open }
        onSelectAccount={ this.onToggleOpen }
        autoGenerateNestedIndicator={ false } />
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
          key={ index } />
      ));
  }

  onToggleOpen () {
    this.setState({ open: !this.state.open });
  }

  onSelectAccount (address) {
    this.props.handleSetSelected(address);
    this.onToggleOpen();
  }

}
