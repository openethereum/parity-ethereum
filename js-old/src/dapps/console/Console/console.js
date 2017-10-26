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
import ReactDOM from 'react-dom';
import { ObjectInspector } from 'react-inspector';

import ConsoleStore from './console.store';
import SettingsStore from '../Settings/settings.store';

import styles from './console.css';

const ICONS = {
  debug: '&nbsp;',
  error: '✖',
  info: 'ℹ',
  input: '&gt;',
  log: '&nbsp;',
  result: '&lt;',
  warn: '⚠'
};

@observer
export default class Console extends Component {
  consoleStore = ConsoleStore.get();
  settingsStore = SettingsStore.get();

  render () {
    return (
      <div ref={ this.setRef }>
        { this.renderResults() }
      </div>
    );
  }

  renderResults () {
    const { logs } = this.consoleStore;

    return logs.map((data, index) => {
      const { type, timestamp } = data;
      const values = this.consoleStore.logValues[index];
      const classes = [ styles.result, styles[type] ];

      return (
        <div
          className={ classes.join(' ') }
          key={ index }
        >
          <span
            className={ styles.type }
            dangerouslySetInnerHTML={ { __html: ICONS[type] || '' } }
          />
          { this.renderTimestamp(timestamp) }
          <span className={ styles.text }>
            {
              values.map((value, valueIndex) => (
                <span
                  className={ styles.token }
                  key={ valueIndex }
                >
                  { this.toString(value) }
                </span>
              ))
            }
          </span>
        </div>
      );
    });
  }

  renderTimestamp (timestamp) {
    const { displayTimestamps } = this.settingsStore;

    if (!displayTimestamps) {
      return null;
    }

    return (
      <span className={ styles.time }>
        { new Date(timestamp).toISOString().slice(11, 23) }
      </span>
    );
  }

  setRef = (node) => {
    const element = ReactDOM.findDOMNode(node);

    this.consoleStore.setNode(element);
  };

  toString (value) {
    if (typeof value === 'string') {
      return value;
    }

    if (value instanceof Error) {
      return value.toString();
    }

    return (
      <ObjectInspector data={ value } />
    );
  }
}
