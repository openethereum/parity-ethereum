import React, { Component } from 'react';

import AccountSummary from './AccountSummary';
import Actions from './Actions';
import { CreateAccount } from '../../modals';
import Tooltip from '../../ui/Tooltip';

import styles from './style.css';

export default class Accounts extends Component {
  static contextTypes = {
    api: React.PropTypes.object
  }

  state = {
    accounts: [],
    newDialog: false
  }

  componentDidMount () {
    // TODO: we should be getting data from a provider
    this._isMounted = true;
    this.retrieveAccounts();
  }

  componentWillUnmount () {
    this._isMounted = false;
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
    if (!this.state.accounts.length) {
      return null;
    }

    const firstTooltip = (
      <Tooltip
        top='80%'
        text='your accounts are visible for easy access, allowing you to edit the meta information, make transfers, view transactions and fund the account' />
    );

    return this.state.accounts.map((account, idx) => {
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

  retrieveAccounts () {
    if (!this._isMounted) {
      return;
    }

    const api = this.context.api;

    Promise
      .all([
        api.personal.listAccounts(),
        api.personal.accountsInfo()
      ])
      .then(([addresses, infos, registryAddress]) => {
        this.setState({
          accounts: addresses
            .filter((address) => infos[address].uuid)
            .map((address) => {
              const info = infos[address];

              return {
                address: address,
                name: info.name,
                uuid: info.uuid,
                meta: info.meta
              };
            })
        });

        setTimeout(() => this.retrieveAccounts(), 2500);
      });
  }
}
