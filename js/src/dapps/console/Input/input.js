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
import AutocompleteStore from '../autocompleteStore';
import EvalStore from '../evalStore';
import InputStore from '../inputStore';

import styles from './input.css';

@observer
export default class Input extends Component {
  autocompleteStore = AutocompleteStore.get();
  evalStore = EvalStore.get();
  inputStore = InputStore.get();

  render () {
    const { input } = this.inputStore;

    return (
      <div className={ styles.container }>
        <Autocomplete />
        <span className={ styles.type }>&gt;</span>
        <input
          autoFocus
          className={ styles.input }
          onChange={ this.handleChange }
          onKeyDown={ this.handleKeyDown }
          ref={ this.setRef }
          type='text'
          value={ input }
        />
      </div>
    );
  }

  handleChange = (event) => {
    const { value } = event.target;

    this.inputStore.updateInput(value);
  };

  handleKeyDown = (event) => {
    const { input } = this.inputStore;
    const codeName = keycode(event);

    // Clear console with CTRL+L
    if (codeName === 'l' && event.ctrlKey) {
      event.preventDefault();
      event.stopPropagation();
      return this.evalStore.clear();
    }

    if (codeName === 'esc') {
      event.preventDefault();
      event.stopPropagation();
      return this.autocompleteStore.hide();
    }

    if (codeName === 'enter') {
      event.preventDefault();
      event.stopPropagation();

      if (this.autocompleteStore.hasSelected) {
        return this.autocompleteStore.select(this.inputStore);
      }

      if (input.length > 0) {
        return this.inputStore.execute();
      }
    }

    if (codeName === 'up') {
      event.preventDefault();
      event.stopPropagation();

      if (this.autocompleteStore.show) {
        return this.autocompleteStore.focus(-1);
      }

      return this.inputStore.selectHistory(-1);
    }

    if (codeName === 'down') {
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
