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

import { action, observable } from 'mobx';
import store from 'store';

import AutocompleteStore from '../Autocomplete/autocomplete.store';
import ConsoleStore from '../Console/console.store';

const LS_HISTORY_KEY = '_console::history';
const MAX_HISTORY_LINES = 5;

let instance;

export default class InputStore {
  @observable input = '';

  autocompleteStore = AutocompleteStore.get();
  consoleStore = ConsoleStore.get();
  history = [];
  historyOffset = null;
  inputNode = null;
  lastInput = '';

  constructor () {
    this.loadHistory();
  }

  static get () {
    if (!instance) {
      instance = new InputStore();
    }

    return instance;
  }

  setInputNode (node) {
    this.inputNode = node;
    this.autocompleteStore.setInputNode(node);
  }

  @action
  updateInput (nextValue = '', updateAutocomplete = true) {
    this.input = nextValue;
    const multilines = nextValue.split('\n').length > 1;

    if (updateAutocomplete && !multilines) {
      this.autocompleteStore.update(nextValue);
    }
  }

  selectHistory (_offset) {
    // No history
    if (this.history.length === 0) {
      return;
    }

    if (this.historyOffset === null) {
      // Can't go down if no history selected
      if (_offset === 1) {
        return;
      }

      this.historyOffset = this.history.length - 1;
      this.lastInput = this.input;
      return this.updateInput(this.history[this.historyOffset], false);
    }

    if (_offset === 1 && this.historyOffset === this.history.length - 1) {
      this.historyOffset = null;
      return this.updateInput(this.lastInput);
    }

    this.historyOffset = Math.max(0, this.historyOffset + _offset);
    const nextInput = this.history[this.historyOffset];

    this.updateInput(nextInput, false);
  }

  execute () {
    const { input } = this;

    this.pushToHistory(input);
    this.consoleStore.evaluate(input);
    this.updateInput('');
    this.historyOffset = null;
    this.autocompleteStore.clearCache();
  }

  pushToHistory (input) {
    // Don't stack twice the same input in
    // history
    if (this.history[this.history.length - 1] !== input) {
      this.history.push(input);
    }

    this.saveHistory();
  }

  loadHistory () {
    this.history = store.get(LS_HISTORY_KEY) || [];
  }

  saveHistory () {
    if (this.history.length > MAX_HISTORY_LINES) {
      this.history = this.history.slice(-1 * MAX_HISTORY_LINES);
    }

    store.set(LS_HISTORY_KEY, this.history.slice());
  }
}
