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

import WatchesStore from '../watchesStore';

import styles from './status.css';

@observer
export default class Status extends Component {
  watchesStore = WatchesStore.get();

  render () {
    return (
      <div className={ styles.container }>
        { this.renderWatches() }
      </div>
    );
  }

  renderWatches () {
    const { watches } = this.watchesStore;
    const names = watches.keys();

    return names.map((name) => {
      const { result, error } = watches.get(name);
      const classes = [ styles.watch ];
      const resultStr = error
        ? error.toString()
        : this.toString(result);

      if (error) {
        classes.push(styles.error);
      }

      return (
        <div
          className={ classes.join(' ') }
          key={ name }
        >
          <span>{ name }</span>
          <span className={ styles.result }>{ resultStr.toString() }</span>
        </div>
      );
    });
  }

  toString (result) {
    if (result === undefined) {
      return 'undefined';
    }

    if (result === null) {
      return 'null';
    }

    if (typeof result.toFormat === 'function') {
      return result.toFormat();
    }

    if (typeof result.toString === 'function') {
      return result.toString();
    }

    return result;
  }
}
