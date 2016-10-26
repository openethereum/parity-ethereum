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

import React, { Component, PropTypes } from 'react';
import { TextField } from 'material-ui';

import IdentityIcon from '../IdentityIcon';
import AccountSelector from '../AccountSelector';

import styles from './accountSelectorText.css';

const NAME_ID = ' ';

export default class AccountSelectorText extends Component {
  static propTypes = {
    accounts: PropTypes.array,
    account: PropTypes.object,
    errorText: PropTypes.string,
    gavBalance: PropTypes.bool,
    anyAccount: PropTypes.bool,
    floatingLabelText: PropTypes.string,
    hintText: PropTypes.string,
    selector: PropTypes.bool,
    onChange: PropTypes.func
  }

  render () {
    const { selector } = this.props;

    return selector
      ? this.renderDropDown()
      : this.renderTextField();
  }

  renderDropDown () {
    const { account, accounts, anyAccount, errorText, gavBalance, hintText, floatingLabelText, onChange } = this.props;

    return (
      <AccountSelector
        anyAccount={ anyAccount }
        gavBalance={ gavBalance }
        accounts={ accounts }
        account={ account }
        errorText={ errorText }
        floatingLabelText={ floatingLabelText }
        hintText={ hintText }
        onSelect={ onChange } />
    );
  }

  renderTextField () {
    const { account, errorText, hintText, floatingLabelText } = this.props;

    return (
      <div className={ styles.addrtext }>
        <TextField
          className={ styles.input }
          autoComplete='off'
          floatingLabelFixed
          floatingLabelText={ floatingLabelText }
          fullWidth
          hintText={ hintText }
          errorText={ errorText }
          name={ NAME_ID }
          id={ NAME_ID }
          value={ account.address || '' }
          onChange={ this.onChangeAddress } />
        { this.renderAddressIcon() }
      </div>
    );
  }

  renderAddressIcon () {
    const { account } = this.props;

    if (!account.address) {
      return null;
    }

    return (
      <div className={ styles.addricon }>
        <IdentityIcon address={ account.address } />
      </div>
    );
  }

  onChangeAddress = (event, address) => {
    const lower = address.toLowerCase();
    const account = this.props.accounts.find((_account) => _account.address.toLowerCase() === lower);

    this.props.onChange(account || { address });
  }
}
