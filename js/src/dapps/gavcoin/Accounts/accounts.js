import React, { Component, PropTypes } from 'react';

import { Chip } from 'material-ui';

const { IdentityIcon } = window.parity.react;

import styles from './accounts.css';

export default class Accounts extends Component {
  static contextTypes = {
    api: PropTypes.object.isRequired,
    instance: PropTypes.object.isRequired
  }

  static propTypes = {
    accounts: PropTypes.array
  }

  render () {
    const has = this._hasAccounts();

    return (
      <div className={ styles.accounts }>
        { has ? this.renderAccounts() : this.renderEmpty() }
      </div>
    );
  }

  renderEmpty () {
    return (
      <div className={ styles.none }>
        You currently do not have any GAVcoin in any of your addresses, buy some
      </div>
    );
  }

  renderAccounts () {
    const { accounts } = this.props;

    return accounts
      .filter((account) => account.hasGav)
      .map((account) => {
        return (
          <Chip
            className={ styles.account }
            key={ account.address }>
            <IdentityIcon
              inline center
              address={ account.address } />
            <span className={ styles.name }>
              { account.name }
            </span>
            <span className={ styles.balance }>
              { account.gavBalance }
            </span>
          </Chip>
        );
      });
  }

  _hasAccounts () {
    const { accounts } = this.props;

    if (!accounts || !accounts.length) {
      return false;
    }

    return accounts.filter((account) => account.hasGav).length !== 0;
  }
}
