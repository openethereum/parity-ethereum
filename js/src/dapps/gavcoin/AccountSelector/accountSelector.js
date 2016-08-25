import React, { Component, PropTypes } from 'react';

import { MenuItem, SelectField } from 'material-ui';

const { IdentityIcon } = window.parity.react;

const NAME_ID = ' ';
let lastSelectedAccount = {};

export default class AccountSelect extends Component {
  static propTypes = {
    accounts: PropTypes.array,
    account: PropTypes.object,
    accountError: PropTypes.string,
    gavBalance: PropTypes.bool,
    onSelect: PropTypes.func
  }

  componentDidMount () {
    this.props.onSelect(lastSelectedAccount);
  }

  render () {
    const { accounts } = this.props;
    const items = accounts.map((account) => {
      const balance = this.props.gavBalance
        ? `${account.gavBalance}GAV`
        : `${account.ethBalance}ΞTH`;
      const identityIcon = (
        <IdentityIcon inline center address={ account.address } />
      );
      const icon = (
        <div className='iconimg'>
          { identityIcon }
        </div>
      );
      const label = (
        <div className='selectaccount'>
          <div className='image'>
            { identityIcon }
          </div>
          <div className='details'>
            <div className='name'>{ account.name }</div>
            <div className='balance'>{ balance }</div>
          </div>
        </div>
      );

      return (
        <MenuItem
          key={ account.address }
          primaryText={ account.name }
          value={ account.address }
          label={ label }
          leftIcon={ icon } />
      );
    });

    return (
      <SelectField
        autoComplete='off'
        floatingLabelFixed
        floatingLabelText='transaction account'
        fullWidth
        hintText='the account the transaction will be made from'
        errorText={ this.props.accountError }
        name={ NAME_ID }
        id={ NAME_ID }
        value={ this.props.account.address }
        onChange={ this.onSelect }>
        { items }
      </SelectField>
    );
  }

  onSelect = (event, idx) => {
    lastSelectedAccount = this.props.accounts[idx] || {};
    this.props.onSelect(lastSelectedAccount);
  }
}
