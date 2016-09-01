import React, { Component, PropTypes } from 'react';

import Actions from './Actions';
import Summary from './Summary';
import { AddressBook, CreateAccount } from '../../modals';
import { Tooltip } from '../../ui';

import styles from './style.css';

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
    return (
      <div>
        { this.renderAddressBook() }
        { this.renderNewDialog() }
        <Actions
          onAddressBook={ this.onAddressBookClick }
          onNewAccount={ this.onNewAccountClick } />
        <div className={ styles.accounts }>
          { this.renderAccounts() }
        </div>
      </div>
    );
  }

  renderAccounts () {
    const { accounts } = this.context;

    if (!accounts) {
      return null;
    }

    const { tokens } = this.state;
    const firstTooltip = (
      <Tooltip
        top='80%'
        text='your accounts are visible for easy access, allowing you to edit the meta information, make transfers, view transactions and fund the account' />
    );

    return accounts
      .filter((acc) => acc.uuid)
      .map((account, idx) => {
        return (
          <div
            className={ styles.account }
            key={ account.address }>
            <Summary
              account={ account }
              tokens={ tokens }>
              { idx === 0 ? firstTooltip : null }
            </Summary>
          </div>
        );
      });
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

  renderAddressBook () {
    const { addressBook } = this.state;

    if (!addressBook) {
      return null;
    }

    return (
      <AddressBook
        onClose={ this.onAddressBookClose } />
    );
  }

  onAddressBookClick = () => {
    this.setState({
      addressBook: !this.state.addressBook
    });
  }

  onNewAccountClick = () => {
    this.setState({
      newDialog: !this.state.newDialog
    });
  }

  onAddressBookClose = () => {
    this.onAddressBookClick();
  }

  onNewAccountClose = () => {
    this.onNewAccountClick();
  }

  onNewAccountUpdate = () => {
  }
}
