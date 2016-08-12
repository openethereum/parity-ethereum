import React, { Component } from 'react';

import AccountSummary from './AccountSummary';
import Actions from './Actions';
import NewAccount from '../NewAccount';

import styles from './style.css';

export default class Accounts extends Component {
  static contextTypes = {
    api: React.PropTypes.object
  }

  state = {
    accounts: [],
    fundDialog: false,
    newDialog: false,
    transferDialog: false
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
          onFundAccount={ this.onFundAccountClick }
          onNewAccount={ this.onNewAccountClick }
          onTransfer={ this.onTransferClick } />
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

  onFundAccountClick = () => {
    this.setState({ fundDialog: !this.state.fundDialog });
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

  onTransferClick = () => {
    this.setState({ transferDialog: !this.state.transferDialog });
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
