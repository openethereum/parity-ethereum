import React, { Component } from 'react';

import AccountSummary from './AccountSummary';
import Actions from './Actions';
import { CreateAccount } from '../../modals';
import Tooltip from '../../ui/Tooltip';

import styles from './style.css';

export default class Accounts extends Component {
  static contextTypes = {
    api: React.PropTypes.object,
    accounts: React.PropTypes.array
  }

  state = {
    newDialog: false
  }

  render () {
    return (
      <div>
        <CreateAccount
          onClose={ this.onNewAccountClose }
          onUpdate={ this.onNewAccountUpdate }
          visible={ this.state.newDialog } />
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
            <AccountSummary
              account={ account }
              tokens={ this.state.tokens }>
              { idx === 0 ? firstTooltip : null }
            </AccountSummary>
          </div>
        );
      });
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
