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

import WatchesStore from './watches.store';

import styles from './watches.css';

@observer
export default class Watches extends Component {
  watchesStore = WatchesStore.get();

  render () {
    return (
      <div className={ styles.container }>
        { this.renderAddWatch() }
        { this.renderWatches() }
      </div>
    );
  }

  renderAddForm () {
    const { showAdd } = this.watchesStore;

    if (!showAdd) {
      return null;
    }

    return (
      <div className={ styles.addForm }>
        <div className={ styles.inputContainer }>
          <input
            className={ styles.input }
            onChange={ this.handleAddNameChange }
            placeholder='Name'
            type='text'
          />
        </div>
        <div className={ styles.inputContainer }>
          <input
            className={ [ styles.input, styles.big ].join(' ') }
            onChange={ this.handleAddFunctionChange }
            placeholder='Function'
            type='text'
          />
        </div>
        <div className={ styles.inputContainer }>
          <input
            className={ styles.input }
            onChange={ this.handleAddContextChange }
            placeholder='Context'
            type='text'
          />
        </div>
        <button
          className={ styles.button }
          onClick={ this.handleAddWatch }
        >
          Add
        </button>
      </div>
    );
  }

  renderAddWatch () {
    const { showAdd } = this.watchesStore;
    const classes = [ styles.add ];

    if (showAdd) {
      classes.push(styles.selected);
    }

    return (
      <div className={ styles.addContainer }>
        { this.renderAddForm() }
        <span
          className={ classes.join(' ') }
          onClick={ this.handleToggleAdd }
        >
          +
        </span>
      </div>
    );
  }

  renderWatches () {
    const { names } = this.watchesStore;

    return names.map((name) => {
      const { result, error } = this.watchesStore.get(name);
      const classes = [ styles.watch ];
      const resultStr = error
        ? error.toString()
        : this.toString(result);

      const onClick = () => this.handleRemoveWatch(name);

      if (error) {
        classes.push(styles.error);
      }

      return (
        <div
          className={ classes.join(' ') }
          key={ name }
        >
          <span
            className={ styles.remove }
            onClick={ onClick }
            title={ `Remove "${name}" watch` }
          >
            <span>✖</span>
          </span>

          <span>{ name }</span>
          <span className={ styles.result }>{ resultStr.toString() }</span>
        </div>
      );
    });
  }

  handleAddFunctionChange = (event) => {
    const { value } = event.target;

    this.watchesStore.updateAddFunction(value);
  };

  handleAddContextChange = (event) => {
    const { value } = event.target;

    this.watchesStore.updateAddContext(value);
  };

  handleAddNameChange = (event) => {
    const { value } = event.target;

    this.watchesStore.updateAddName(value);
  };

  handleAddWatch = () => {
    this.watchesStore.addWatch();
  };

  handleRemoveWatch = (name) => {
    this.watchesStore.remove(name);
  };

  handleToggleAdd = () => {
    this.watchesStore.toggleAdd();
  };

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
