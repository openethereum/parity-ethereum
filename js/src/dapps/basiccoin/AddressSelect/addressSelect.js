// Copyright 2015-2017 Parity Technologies (UK) Ltd.
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

import { api } from '../parity';
import styles from './addressSelect.css';

export default class AddressSelect extends Component {
  static contextTypes = {
    accounts: PropTypes.object.isRequired
  }

  static propTypes = {
    addresses: PropTypes.array.isRequired,
    onChange: PropTypes.func.isRequired
  }

  state = {
    selected: null
  }

  componentDidMount () {
    const { addresses } = this.props;

    this.onChange({
      target: {
        value: addresses[0]
      }
    });
  }

  componentWillReceiveProps (newProps) {
    const { addresses } = this.props;
    let changed = addresses.length !== newProps.addresses.length;

    if (!changed) {
      changed = addresses.filter((address, index) => newProps.addresses[index] !== address).length;
    }

    if (changed) {
      this.onChange({ target: { value: newProps.addresses[0] } });
    }
  }

  render () {
    const { addresses } = this.props;
    const { selectedAddress } = this.state;
    const style = {
      background: `rgba(255, 255, 255, 0.75) url(${api.util.createIdentityImg(selectedAddress, 3)}) no-repeat 98% center`
    };

    return (
      <select
        className={ styles.iconMenu }
        style={ style }
        onChange={ this.onChange }
      >
        { addresses.map(this.renderOption) }
      </select>
    );
  }

  renderOption = (address) => {
    const { accounts } = this.context;
    const account = accounts[address];

    return (
      <option
        key={ account.address }
        value={ account.address }
      >
        { account.name }
      </option>
    );
  }

  onChange = (event) => {
    this.setState({ selectedAddress: event.target.value });
    this.props.onChange(event);
  }
}
