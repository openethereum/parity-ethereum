import React, { Component } from 'react';

import AccountSummary from './AccountSummary';
import Actions from './Actions';
import { NewAccount } from '../../dialogs';

import styles from './style.css';

export default class Accounts extends Component {
  static contextTypes = {
    api: React.PropTypes.object
  }

  state = {
    accounts: [],
    newDialog: false
  }

  componentWillMount () {
    this.retrieveAccounts();
  }

  render () {
    return (
      <div>
        <NewAccount
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
    if (!this.state.accounts.length) {
      return null;
    }

    return this.state.accounts.map((account) => {
      return (
        <div
          className={ styles.account }
          key={ account.address }>
          <AccountSummary
            account={ account } />
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
    this.retrieveAccounts();
  }

  retrieveAccounts () {
    const api = this.context.api;

    Promise
      .all([
        api.personal.listAccounts(),
        api.personal.accountsInfo()
      ])
      .then(([addresses, infos]) => {
        this.setState({
          accounts: addresses.map((address) => {
            const info = infos[address];

            return {
              address: address,
              name: info.name,
              uuid: info.uuid,
              meta: info.meta
            };
          })
        });
      });
  }
}
