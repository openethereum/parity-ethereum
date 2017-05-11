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

import AutocompleteStore from '../Autocomplete/autocomplete.store';
import { evaluate } from '../utils';

let instance;

export default class ConsoleStore {
  @observable logs = [];

  autocompleteStore = AutocompleteStore.get();
  logValues = [];
  node = null;

  constructor () {
    this.attachConsole();
  }

  static get () {
    if (!instance) {
      instance = new ConsoleStore();
    }

    return instance;
  }

  attachConsole () {
    ['debug', 'error', 'info', 'log', 'warn'].forEach((level) => {
      const old = window.console[level].bind(window.console);

      window.console[level] = (...args) => {
        old(...args);
        this.log({ type: level, values: args });
      };
    });
  }

  @action
  clear () {
    this.logs = [];
    this.logValues = [];
  }

  evaluate (input) {
    this.log({ type: 'input', value: input });

    setTimeout(() => {
      const { result, error } = evaluate(input);
      let value = error || result;
      const type = error
        ? 'error'
        : 'result';

      if (typeof value === 'string') {
        value = `"${value}"`;
      }

      if (value && typeof value === 'object' && typeof value.then === 'function') {
        return value
          .then((result) => {
            this.log({ type: 'result', value: result });
          })
          .catch((error) => {
            this.log({ type: 'error', value: error });
          });
      }

      this.log({ type, value });
    });
  }

  @action
  log ({ type, value, values }) {
    this.logs.push({
      type,
      timestamp: Date.now()
    });

    if (values) {
      this.logValues.push(values);
    } else {
      this.logValues.push([ value ]);
    }

    this.autocompleteStore.setPosition();
    this.scroll();
  }

  setNode (node) {
    this.node = node;
    this.scroll();
  }

  scroll () {
    if (!this.node) {
      return;
    }

    setTimeout(() => {
      if (this.node.children.length === 0) {
        return;
      }

      // Scroll to the last child
      this.node
        .children[this.node.children.length - 1]
        .scrollIntoView(false);
    }, 50);
  }
}
