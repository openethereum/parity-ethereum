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
import { MenuItem } from 'material-ui';

import IdentityIcon from '../../IdentityIcon';
import Select from '../Select';

import styles from './addressSelect.css';

export default class AddressSelect extends Component {
  static propTypes = {
    disabled: PropTypes.bool,
    accounts: PropTypes.object,
    contacts: PropTypes.object,
    label: PropTypes.string,
    hint: PropTypes.string,
    error: PropTypes.string,
    value: PropTypes.string,
    tokens: PropTypes.object,
    onChange: PropTypes.func.isRequired
  }

  render () {
    const { disabled, error, hint, label, value } = this.props;

    return (
      <Select
        disabled={ disabled }
        label={ label }
        hint={ hint }
        error={ error }
        value={ value }
        onChange={ this.onChange }>
        { this.renderSelectEntries() }
      </Select>
    );
  }

  renderSelectEntries () {
    const { accounts, contacts } = this.props;
    const entries = Object.values(Object.assign({}, accounts || {}, contacts || {}));

    if (!entries.length) {
      return null;
    }

    return entries.map(this.renderSelectEntry);
  }

  renderSelectEntry = (entry) => {
    const item = (
      <div className={ styles.account }>
        <div className={ styles.image }>
          <IdentityIcon
            inline center
            address={ entry.address } />
        </div>
        <div className={ styles.details }>
          <div className={ styles.name }>
            { entry.name || 'Unnamed' }
          </div>
        </div>
      </div>
    );

    return (
      <MenuItem
        key={ entry.address }
        value={ entry.address }
        label={ item }>
        { item }
      </MenuItem>
    );
  }

  onChange = (event, idx, value) => {
    this.props.onChange(event, value);
  }
}
