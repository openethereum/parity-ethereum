// Copyright 2015, 2016 Parity Technologies (UK) Ltd.
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
import { isEqual, pick } from 'lodash';

import AutoComplete from '../AutoComplete';
import IdentityIcon from '../../IdentityIcon';
import IdentityName from '../../IdentityName';

import { fromWei } from '~/api/util/wei';

import styles from './addressSelect.css';

export default class AddressSelect extends Component {
  static contextTypes = {
    api: PropTypes.object.isRequired
  }

  static propTypes = {
    onChange: PropTypes.func.isRequired,

    accounts: PropTypes.object,
    allowInput: PropTypes.bool,
    balances: PropTypes.object,
    contacts: PropTypes.object,
    contracts: PropTypes.object,
    disabled: PropTypes.bool,
    error: PropTypes.string,
    hint: PropTypes.string,
    label: PropTypes.string,
    tokens: PropTypes.object,
    value: PropTypes.string,
    wallets: PropTypes.object
  }

  state = {
    autocompleteEntries: [],
    entries: {},
    addresses: [],
    value: ''
  }

  // Cache autocomplete items
  items = {}

  entriesFromProps (props = this.props) {
    const { accounts = {}, contacts = {}, contracts = {}, wallets = {} } = props;

    const autocompleteEntries = [].concat(
      Object.values(wallets),
      'divider',
      Object.values(accounts),
      'divider',
      Object.values(contacts),
      'divider',
      Object.values(contracts)
    );

    const entries = {
      ...wallets,
      ...accounts,
      ...contacts,
      ...contracts
    };

    return { autocompleteEntries, entries };
  }

  shouldComponentUpdate (nextProps, nextState) {
    const keys = [ 'error', 'value' ];

    const prevValues = pick(this.props, keys);
    const nextValues = pick(nextProps, keys);

    return !isEqual(prevValues, nextValues);
  }

  componentWillMount () {
    const { value } = this.props;
    const { entries, autocompleteEntries } = this.entriesFromProps();
    const addresses = Object.keys(entries).sort();

    this.setState({ autocompleteEntries, entries, addresses, value });
  }

  componentWillReceiveProps (newProps) {
    if (newProps.value !== this.props.value) {
      this.setState({ value: newProps.value });
    }
  }

  render () {
    const { allowInput, disabled, error, hint, label } = this.props;
    const { autocompleteEntries, value } = this.state;

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
          entries={ autocompleteEntries }
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
    const { address, name } = entry;

    const _balance = this.getBalance(address);
    const balance = _balance ? _balance.toNumber() : _balance;

    if (!this.items[address] || this.items[address].balance !== balance) {
      this.items[address] = {
        text: name && name.toUpperCase() || address,
        value: this.renderMenuItem(address),
        address, balance
      };
    }

    return this.items[address];
  }

  getBalance (address) {
    const { balances = {} } = this.props;
    const balance = balances[address];

    if (!balance) {
      return null;
    }

    const ethToken = balance.tokens.find((tok) => tok.token && tok.token.tag && tok.token.tag.toLowerCase() === 'eth');

    if (!ethToken) {
      return null;
    }

    return ethToken.value;
  }

  renderBalance (address) {
    const balance = this.getBalance(address);
    const value = fromWei(balance);

    return (
      <div className={ styles.balance }>
        { value.toFormat(3) }<small> { 'ETH' }</small>
      </div>
    );
  }

  renderMenuItem (address) {
    const balance = this.props.balances
      ? this.renderBalance(address)
      : null;

    const item = (
      <div className={ styles.account }>
        <IdentityIcon
          className={ styles.image }
          inline center
          address={ address } />
        <IdentityName
          className={ styles.name }
          address={ address } />
        { balance }
      </div>
    );

    return (
      <MenuItem
        className={ styles.menuItem }
        key={ address }
        value={ address }
        label={ item }>
        { item }
      </MenuItem>
    );
  }

  getSearchText () {
    const entry = this.getEntry();

    return entry && entry.name
      ? entry.name.toUpperCase()
      : this.state.value;
  }

  getEntry () {
    const { entries, value } = this.state;
    return value ? entries[value] : null;
  }

  handleFilter = (searchText, name, item) => {
    const { address } = item;
    const entry = this.state.entries[address];
    const lowCaseSearch = (searchText || '').toLowerCase();

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
