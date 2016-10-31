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

import AutoComplete from '../AutoComplete';
import IdentityIcon from '../../IdentityIcon';
import IdentityName from '../../IdentityName';

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

  state = {
    entries: {},
    value: ''
  }

  componentWillMount () {
    const { accounts, contacts, value } = this.props;
    const entries = Object.assign({}, accounts || {}, contacts || {});
    this.setState({ entries, value });
  }

  componentWillReceiveProps (newProps) {
    const { accounts, contacts } = newProps;
    const entries = Object.assign({}, accounts || {}, contacts || {});
    this.setState({ entries });
  }

  render () {
    const { disabled, error, hint, label } = this.props;
    const { entries } = this.state;
    const value = this.getSearchText();

    return (
      <div className={ styles.container }>
        <AutoComplete
          className={ (error || !value) ? '' : styles.paddedInput }
          disabled={ disabled }
          label={ label }
          hint={ hint ? `search for ${hint}` : 'search for an address' }
          error={ error }
          onChange={ this.onChange }
          value={ value }
          filter={ this.handleFilter }
          entries={ entries }
          entry={ this.getEntry() || {} }
          renderItem={ this.renderItem }
        />

        { this.renderIdentityIcon(value) }
      </div>
    );
  }

  renderIdentityIcon (inputValue) {
    const { error, value } = this.props;

    if (error || !inputValue) {
      return null;
    }

    return (
      <IdentityIcon
        className={ styles.icon }
        inline center
        address={ value } />
    );
  }

  renderItem = (entry) => {
    return {
      text: entry.address,
      value: this.renderSelectEntry(entry)
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
    if (!entry) return '';

    return entry.name ? entry.name.toUpperCase() : '';
  }

  getEntry () {
    const { value } = this.props;
    if (!value) return '';

    const { entries } = this.state;
    return entries[value];
  }

  handleFilter = (searchText, address) => {
    const entry = this.state.entries[address];
    const lowCaseSearch = searchText.toLowerCase();

    return [ entry.name, entry.address ]
      .some(text => text.toLowerCase().indexOf(lowCaseSearch) !== -1);
  }

  onChange = (entry, empty) => {
    const address = entry && entry.address
      ? entry.address
      : (empty ? '' : this.state.value);

    this.props.onChange(null, address);
  }
}
