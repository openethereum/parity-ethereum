import React, { Component, PropTypes } from 'react';

import Actions from './Actions';
import Summary from './Summary';
import { CreateAccount } from '../../modals';
import Tooltip from '../../ui/Tooltip';

import styles from './style.css';

export default class Accounts extends Component {
  static contextTypes = {
    api: PropTypes.object,
    accounts: PropTypes.array
  }

  state = {
    newDialog: false
  }

  render () {
    return (
      <div>
        { this.renderNewDialog() }
        <Actions
          onNewAccount={ this.onNewAccountClick } />
        <div className={ styles.accounts }>
          { this.renderAccounts() }
        </div>
      </div>
    );
  }

  renderAccounts () {
    if (!this.context.accounts) {
      return null;
    }

    const firstTooltip = (
      <Tooltip
        top='80%'
        text='your accounts are visible for easy access, allowing you to edit the meta information, make transfers, view transactions and fund the account' />
    );

    return this.context.accounts
      .filter((acc) => acc.uuid)
      .map((account, idx) => {
        return (
          <div
            className={ styles.account }
            key={ account.address }>
            <Summary
              account={ account }
              tokens={ this.state.tokens }>
              { idx === 0 ? firstTooltip : null }
            </Summary>
          </div>
        );
      });
  }

  renderNewDialog () {
    if (!this.state.newDialog) {
      return null;
    }

    return (
      <CreateAccount
        onClose={ this.onNewAccountClose }
        onUpdate={ this.onNewAccountUpdate } />
    );
  }

  onNewAccountClick = () => {
    this.setState({ newDialog: !this.state.newDialog });
  }

  onNewAccountClose = () => {
    this.onNewAccountClick();
  }

  onNewAccountUpdate = () => {
  }
}
