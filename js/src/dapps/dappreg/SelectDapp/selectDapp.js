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

import React, { Component } from 'react';
import { observer } from 'mobx-react';

import DappsStore from '../dappsStore';

import Input from '../Input';

@observer
export default class SelectDapp extends Component {
  dappsStore = DappsStore.instance();

  render () {
    if (this.dappsStore.isNew) {
      return (
        <Input
          hint='...'
          label='Application Id, the unique assigned identifier'
        >
          <input value={ this.dappsStore.wipApp.id } readOnly />
        </Input>
      );
    }

    if (!this.dappsStore.currentApp) {
      return null;
    }

    let overlayImg = null;

    if (this.dappsStore.currentApp.imageHash) {
      overlayImg = (
        <img src={ `/api/content/${this.dappsStore.currentApp.imageHash.substr(2)}` } />
      );
    }

    return (
      <Input
        hint={ this.dappsStore.currentApp.id }
        label='Application, the actual application details to show below'
        overlay={ overlayImg }
      >
        <select
          disabled={ this.dappsStore.isEditing }
          value={ this.dappsStore.currentApp.id }
          onChange={ this.onSelect }
        >
          { this.renderOptions() }
        </select>
      </Input>
    );
  }

  renderOptions () {
    return this.dappsStore.apps.map((app) => {
      return (
        <option
          value={ app.id }
          key={ app.id }
        >
          { app.name }
        </option>
      );
    });
  }

  onSelect = (event) => {
    this.dappsStore.setCurrentApp(event.target.value);
  }
}
