// Copyright 2015, 2016 Ethcore (UK) Ltd.
// This file is part of Parity.

// Parity is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Parity is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Parity.  If not, see <http://www.gnu.org/licenses/>.

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
