import React, { Component, PropTypes } from 'react';

import { Chip } from 'material-ui';

const { IdentityIcon } = window.parity.react;

export default class Accounts extends Component {
  static contextTypes = {
    api: PropTypes.object,
    instance: PropTypes.object
  }

  static propTypes = {
    accounts: PropTypes.array
  }

  render () {
    const has = this._hasAccounts();

    return (
      <div className='accounts'>
        { has ? this.renderAccounts() : this.renderEmpty() }
      </div>
    );
  }

  renderEmpty () {
    return (
      <div className='none'>
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
            className='account'
            key={ account.address }>
            <IdentityIcon
              inline center
              address={ account.address } />
            <span className='name'>{ account.name }</span>
            <span className='balance'>{ account.gavBalance }</span>
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
