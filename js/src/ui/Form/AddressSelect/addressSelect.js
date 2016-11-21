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
import { isEqual } from 'lodash';

import AutoComplete from '../AutoComplete';
import IdentityIcon from '../../IdentityIcon';
import IdentityName from '../../IdentityName';

import styles from './addressSelect.css';

export default class AddressSelect extends Component {
  static contextTypes = {
    api: PropTypes.object.isRequired
  }

  static propTypes = {
    disabled: PropTypes.bool,
    accounts: PropTypes.object,
    contacts: PropTypes.object,
    contracts: PropTypes.object,
    label: PropTypes.string,
    hint: PropTypes.string,
    error: PropTypes.string,
    value: PropTypes.string,
    tokens: PropTypes.object,
    onChange: PropTypes.func.isRequired,
    allowInput: PropTypes.bool
  }

  state = {
    entries: {},
    addresses: [],
    value: ''
  }

  entriesFromProps (props = this.props) {
    const { accounts, contacts, contracts } = props;
    const entries = Object.assign({}, accounts || {}, contacts || {}, contracts || {});
    return entries;
  }

  componentWillMount () {
    const { value } = this.props;
    const entries = this.entriesFromProps();
    const addresses = Object.keys(entries).sort();

    this.setState({ entries, addresses, value });
  }

  componentWillReceiveProps (newProps) {
    const entries = this.entriesFromProps();
    const addresses = Object.keys(entries).sort();

    if (!isEqual(addresses, this.state.addresses)) {
      this.setState({ entries, addresses });
    }

    if (newProps.value !== this.props.value) {
      this.setState({ value: newProps.value });
    }
  }

  render () {
    const { allowInput, disabled, error, hint, label } = this.props;
    const { entries, value } = this.state;

    const searchText = this.getSearchText();
    const icon = this.renderIdentityIcon(value);

    return (
      <div className={ styles.container }>
        <AutoComplete
          className={ !icon ? '' : styles.paddedInput }
          disabled={ disabled }
          label={ label }
          hint={ hint ? `search for ${hint}` : 'search for an address' }
          error={ error }
          onChange={ this.onChange }
          onBlur={ this.onBlur }
          onUpdateInput={ allowInput && this.onUpdateInput }
          value={ searchText }
          filter={ this.handleFilter }
          entries={ entries }
          entry={ this.getEntry() || {} }
          renderItem={ this.renderItem }
        />
        { icon }
      </div>
    );
  }

  renderIdentityIcon (inputValue) {
    const { error, value, label } = this.props;

    if (error || !inputValue || value.length !== 42) {
      return null;
    }

    const classes = [ styles.icon ];

    if (!label) {
      classes.push(styles.noLabel);
    }

    return (
      <IdentityIcon
        className={ classes.join(' ') }
        inline center
        address={ value } />
    );
  }

  renderItem = (entry) => {
    return {
      text: entry.name && entry.name.toUpperCase() || entry.address,
      value: this.renderSelectEntry(entry),
      address: entry.address
    };
  }

  renderSelectEntry = (entry) => {
    const item = (
      <div className={ styles.account }>
        <IdentityIcon
          className={ styles.image }
          inline center
          address={ entry.address } />
        <IdentityName
          className={ styles.name }
          address={ entry.address } />
      </div>
    );

    return (
      <MenuItem
        className={ styles.menuItem }
        key={ entry.address }
        value={ entry.address }
        label={ item }>
        { item }
      </MenuItem>
    );
  }

  getSearchText () {
    const entry = this.getEntry();
    const { value } = this.state;

    return entry && entry.name
      ? entry.name.toUpperCase()
      : value;
  }

  getEntry () {
    const { entries, value } = this.state;
    return value ? entries[value] : null;
  }

  handleFilter = (searchText, name, item) => {
    const { address } = item;
    const entry = this.state.entries[address];
    const lowCaseSearch = searchText.toLowerCase();

    return [entry.name, entry.address]
      .some(text => text.toLowerCase().indexOf(lowCaseSearch) !== -1);
  }

  onChange = (entry, empty) => {
    const { allowInput } = this.props;
    const { value } = this.state;

    const address = entry && entry.address
      ? entry.address
      : ((empty && !allowInput) ? '' : value);

    this.props.onChange(null, address);
  }

  onUpdateInput = (query, choices) => {
    const { api } = this.context;

    const address = query.trim();

    if (!/^0x/.test(address) && api.util.isAddressValid(`0x${address}`)) {
      const checksumed = api.util.toChecksumAddress(`0x${address}`);
      return this.props.onChange(null, checksumed);
    }

    this.props.onChange(null, address);
  };
}
