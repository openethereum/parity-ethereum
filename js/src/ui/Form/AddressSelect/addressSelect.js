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
    entries: {}
  }

  componentWillMount () {
    const { accounts, contacts } = this.props;
    const entries = Object.assign({}, accounts || {}, contacts || {});
    this.setState({ entries });
  }

  componentWillReceiveProps (newProps) {
    const { accounts, contacts } = newProps;
    const entries = Object.assign({}, accounts || {}, contacts || {});
    this.setState({ entries });
  }

  render () {
    const { disabled, error, hint, label } = this.props;
    const { entries } = this.state;

    return (
      <div className={ styles.container }>
        <AutoComplete
          className={ error ? '' : styles.paddedInput }
          disabled={ disabled }
          label={ label }
          hint={ `search for ${hint}` }
          error={ error }
          onChange={ this.onChange }
          value={ this.getSearchText() }
          filter={ this.handleFilter }
          entries={ entries }
          renderItem={ this.renderItem }
        />

        { this.renderIdentityIcon() }
      </div>
    );
  }

  renderIdentityIcon () {
    if (this.props.error) {
      return null;
    }

    const { value } = this.props;

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
        <div className={ styles.image }>
          <IdentityIcon
            inline center
            address={ entry.address } />
        </div>
        <div className={ styles.details }>
          <div className={ styles.name }>
            <IdentityName address={ entry.address } />
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

  getSearchText () {
    const { value } = this.props;
    if (!value) return '';

    const { entries } = this.state;
    const entry = entries[value];
    if (!entry) return '';

    return entry.name ? entry.name.toUpperCase() : '';
  }

  handleFilter = (searchText, address) => {
    const entry = this.state.entries[address];
    const lowCaseSearch = searchText.toLowerCase();

    return [ entry.name, entry.address ]
      .some(text => text.toLowerCase().indexOf(lowCaseSearch) !== -1);
  }

  onChange = (entry) => {
    const address = entry ? entry.address : '';
    this.props.onChange(null, address);
  }
}
