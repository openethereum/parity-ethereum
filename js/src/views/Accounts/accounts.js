import React, { Component, PropTypes } from 'react';
import { FlatButton } from 'material-ui';
import ContentAdd from 'material-ui/svg-icons/content/add';

import List from './List';
import { CreateAccount } from '../../modals';
import { Actionbar, Tooltip } from '../../ui';

import styles from './accounts.css';

export default class Accounts extends Component {
  static contextTypes = {
    api: PropTypes.object,
    accounts: PropTypes.array
  }

  state = {
    addressBook: false,
    newDialog: false
  }

  render () {
    const { accounts } = this.context;

    return (
      <div className={ styles.accounts }>
        { this.renderNewDialog() }
        { this.renderActionbar() }
        <List accounts={ accounts } />
        <Tooltip
          className={ styles.accountTooltip }
          text='your accounts are visible for easy access, allowing you to edit the meta information, make transfers, view transactions and fund the account' />
      </div>
    );
  }

  renderActionbar () {
    const buttons = [
      <FlatButton
        key='newAccount'
        icon={ <ContentAdd /> }
        label='new account'
        primary
        onTouchTap={ this.onNewAccountClick } />
    ];

    return (
      <Actionbar
        className={ styles.toolbar }
        title='Accounts Overview'
        buttons={ buttons }>
        <Tooltip
          className={ styles.toolbarTooltip }
          right
          text='actions relating to the current view are available on the toolbar for quick access, be it for performing actions or creating a new item' />
      </Actionbar>
    );
  }

  renderNewDialog () {
    const { newDialog } = this.state;

    if (!newDialog) {
      return null;
    }

    return (
      <CreateAccount
        onClose={ this.onNewAccountClose }
        onUpdate={ this.onNewAccountUpdate } />
    );
  }

  onNewAccountClick = () => {
    this.setState({
      newDialog: !this.state.newDialog
    });
  }

  onNewAccountClose = () => {
    this.onNewAccountClick();
  }

  onNewAccountUpdate = () => {
  }
}
