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

import React, { Component } from 'react';
import { observer } from 'mobx-react';

import Store from '../store';

@observer
export default class SelectAccount extends Component {
  store = Store.instance();

  render () {
    return (
      <select
        value={ this.store.currentAccount.address }
        onChange={ this.onSelect }>
        { this.renderOptions() }
      </select>
    );
  }

  renderOptions () {
    return this.store.accounts.map((account) => {
      return (
        <option value={ account.address } key={ account.address }>
          { account.name }
        </option>
      );
    });
  }

  onSelect = (event) => {
    this.store.setCurrentAccount(event.target.value);
  }
}
