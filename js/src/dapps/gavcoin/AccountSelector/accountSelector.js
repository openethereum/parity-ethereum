import BigNumber from 'bignumber.js';
import React, { Component, PropTypes } from 'react';
import { MenuItem, SelectField } from 'material-ui';

import AccountItem from './AccountItem';

const NAME_ID = ' ';
let lastSelectedAccount = {};

export default class AccountSelect extends Component {
  static propTypes = {
    accounts: PropTypes.array,
    account: PropTypes.object,
    anyAccount: PropTypes.bool,
    gavBalance: PropTypes.bool,
    onSelect: PropTypes.func,
    errorText: PropTypes.string,
    floatingLabelText: PropTypes.string,
    hintText: PropTypes.string
  }

  componentDidMount () {
    this.props.onSelect(lastSelectedAccount);
  }

  render () {
    const { account, accounts, anyAccount, errorText, floatingLabelText, gavBalance, hintText } = this.props;

    return (
      <SelectField
        autoComplete='off'
        floatingLabelFixed
        floatingLabelText={ floatingLabelText }
        fullWidth
        hintText={ hintText }
        errorText={ errorText }
        name={ NAME_ID }
        id={ NAME_ID }
        value={ account }
        onChange={ this.onSelect }>
        { renderAccounts(accounts, { anyAccount, gavBalance }) }
      </SelectField>
    );
  }

  onSelect = (event, idx, account) => {
    lastSelectedAccount = account || {};
    this.props.onSelect(lastSelectedAccount);
  }
}

function isPositive (numberStr) {
  return new BigNumber(numberStr.replace(',', '')).gt(0);
}

export function renderAccounts (accounts, options = {}) {
  return accounts
    .filter((account) => {
      if (options.anyAccount) {
        return true;
      }

      if (account.uuid) {
        return isPositive(account[options.gavBalance ? 'gavBalance' : 'ethBalance']);
      }

      return false;
    })
    .map((account) => {
      const item = (
        <AccountItem
          account={ account }
          key={ account.address }
          gavBalance={ options.gavBalance || false } />
      );

      return (
        <MenuItem
          key={ account.address }
          value={ account }
          label={ item }>
          { item }
        </MenuItem>
      );
    });
}
