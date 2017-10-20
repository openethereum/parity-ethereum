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

import { observer } from 'mobx-react';
import React, { Component } from 'react';

import SettingsStore from './settings.store';

import styles from './settings.css';

@observer
export default class Settings extends Component {
  settingsStore = SettingsStore.get();

  render () {
    const { displayTimestamps, executeOnEnter } = this.settingsStore;

    return (
      <div className={ styles.container }>
        <div className={ styles.option }>
          <input
            checked={ executeOnEnter }
            id='executeOnEnter'
            onChange={ this.handleExecuteOnEnterChange }
            type='checkbox'
          />
          <label htmlFor='executeOnEnter'>
            Execute on <code>Enter</code>
          </label>
        </div>
        <div className={ styles.option }>
          <input
            checked={ displayTimestamps }
            id='displayTimestamps'
            onChange={ this.handleDisplayTimestampsChange }
            type='checkbox'
          />
          <label htmlFor='displayTimestamps'>
            Show timestamps
          </label>
        </div>
      </div>
    );
  }

  handleDisplayTimestampsChange = (event) => {
    const { checked } = event.target;

    this.settingsStore.setDisplayTimestamps(checked);
  };

  handleExecuteOnEnterChange = (event) => {
    const { checked } = event.target;

    this.settingsStore.setExecuteOnEnter(checked);
  };
}
