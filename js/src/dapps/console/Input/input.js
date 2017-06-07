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

import keycode from 'keycode';
import { observer } from 'mobx-react';
import React, { Component } from 'react';
import ReactDOM from 'react-dom';

import Autocomplete from '../Autocomplete';

import AutocompleteStore from '../Autocomplete/autocomplete.store';
import ConsoleStore from '../Console/console.store';
import InputStore from './input.store';
import SettingsStore from '../Settings/settings.store';

import styles from './input.css';

@observer
export default class Input extends Component {
  autocompleteStore = AutocompleteStore.get();
  consoleStore = ConsoleStore.get();
  inputStore = InputStore.get();
  settingsStore = SettingsStore.get();

  render () {
    const { input } = this.inputStore;

    return (
      <div className={ styles.container }>
        <Autocomplete />
        <span className={ styles.type }>&gt;</span>
        <div className={ styles.inputContainer }>
          <textarea
            autoFocus
            className={ styles.input }
            onChange={ this.handleChange }
            onKeyDown={ this.handleKeyDown }
            ref={ this.setRef }
            rows={ input.split('\n').length }
            type='text'
            value={ input }
          />
        </div>
      </div>
    );
  }

  handleChange = (event) => {
    const { value } = event.target;

    this.inputStore.updateInput(value);
  };

  handleKeyDown = (event) => {
    const { executeOnEnter } = this.settingsStore;
    const { input } = this.inputStore;
    const codeName = keycode(event);
    const multilines = input.split('\n').length > 1;

    // Clear console with CTRL+L
    if (codeName === 'l' && event.ctrlKey) {
      event.preventDefault();
      event.stopPropagation();
      return this.consoleStore.clear();
    }

    if (codeName === 'esc') {
      event.preventDefault();
      event.stopPropagation();
      return this.autocompleteStore.hide();
    }

    if (codeName === 'enter') {
      if (event.shiftKey) {
        return;
      }

      // If not execute on enter: execute on
      // enter + CTRL
      if (!executeOnEnter && !event.ctrlKey) {
        return;
      }

      event.preventDefault();
      event.stopPropagation();

      if (this.autocompleteStore.hasSelected) {
        return this.autocompleteStore.select(this.inputStore);
      }

      if (input.length > 0) {
        return this.inputStore.execute();
      }
    }

    if (codeName === 'up' && !multilines) {
      event.preventDefault();
      event.stopPropagation();

      if (this.autocompleteStore.show) {
        return this.autocompleteStore.focus(-1);
      }

      return this.inputStore.selectHistory(-1);
    }

    if (codeName === 'down' && !multilines) {
      event.preventDefault();
      event.stopPropagation();

      if (this.autocompleteStore.show) {
        return this.autocompleteStore.focus(1);
      }

      return this.inputStore.selectHistory(1);
    }

    if (codeName === 'left' && this.autocompleteStore.show) {
      return this.autocompleteStore.hide();
    }

    if (codeName === 'right' && this.autocompleteStore.show) {
      event.preventDefault();
      event.stopPropagation();
      return this.autocompleteStore.select(this.inputStore);
    }
  };

  setRef = (node) => {
    this.inputStore.setInputNode(ReactDOM.findDOMNode(node));
  };
}
