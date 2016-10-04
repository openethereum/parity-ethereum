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

  render () {
    const { addresses } = this.props;

    return (
      <select
        className={ styles.iconMenu }
        onChange={ this.onChange }>
        { addresses.map(this.renderOption) }
      </select>
    );
  }

  renderOption = (address) => {
    const { accounts } = this.context;
    const account = accounts[address];
    const style = { background: `transparent url(${api.util.createIdentityImg(account.address, 3)}) no-repeat left center` };

    return (
      <option
        key={ account.address }
        style={ style }
        value={ account.address }>
        { account.name }
      </option>
    );
  }

  onChange = (event) => {
    this.setState({ selected: event.target.value });
    this.props.onChange(event);
  }
}
