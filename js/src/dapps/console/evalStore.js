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

/* eslint-disable no-eval */

import { action, observable, transaction } from 'mobx';

let instance;

export default class EvalStore {
  @observable input = '';
  @observable inputs = [];
  @observable logs = [];
  @observable autocompletes = [];
  @observable showAutocomplete = true;

  lastObject = null;
  lastObjectPropertyNames = [];
  inputNode = null;

  constructor () {
    this.attachConsole();
  }

  static get () {
    if (!instance) {
      instance = new EvalStore();
    }

    return instance;
  }

  setInputNode (node) {
    this.inputNode = node;
  }

  attachConsole () {
    ['debug', 'error', 'info', 'log', 'warn'].forEach((level) => {
      const old = window.console[level].bind(window.console);

      window.console[level] = (...args) => {
        old(...args);
        this.log({ type: level, value: args });
      };
    });
  }

  @action
  clearLogs () {
    this.logs = [];
  }

  @action
  updateInput (nextValue = '', updateAutocomplete = true) {
    transaction(() => {
      this.input = nextValue;

      if (updateAutocomplete) {
        this.updateAutocomplete();
      }
    });
  }

  @action
  setAutocomplete (autocompletes) {
    this.showAutocomplete = true;
    this.autocompletes = autocompletes;
  }

  @action
  log ({ type, value }) {
    this.logs.push({
      type, value,
      timestamp: Date.now()
    });
  }

  @action
  logInput (input) {
    transaction(() => {
      this.inputs.push(input);
      this.log({ type: 'input', value: input });
    });
  }

  @action
  hideAutocomplete () {
    this.showAutocomplete = false;
    this.focusOnInput();
  }

  focusOnInput () {
    if (!this.inputNode) {
      return;
    }

    setTimeout(() => {
      this.inputNode.focus();
    });
  }

  selectAutocomplete (autocomplete) {
    const { name } = autocomplete;
    const { input } = this;

    if (!input || input.length === 0) {
      this.hideAutocomplete();
      return this.updateInput(name, false);
    }

    const objects = input.split('.');

    objects.pop();
    const nextInput = objects.concat(name).join('.');

    this.hideAutocomplete();
    return this.updateInput(nextInput, false);
  }

  updateAutocomplete () {
    const { input } = this;

    if (!input || input.length === 0) {
      return this.setAutocomplete([]);
    }

    const objects = input.split('.');
    const suffix = objects.pop().toLowerCase();
    const prefix = objects.join('.');
    const object = prefix.length > 0
      ? prefix
      : 'window';

    if (object !== this.lastObject) {
      const evalResult = evaluate(object);

      if (evalResult.error) {
        this.lastObjectProperties = [];
      } else {
        this.lastObjectProperties = getAllProperties(evalResult.result);
      }

      this.lastObject = object;
    }

    const autocompletes = this.lastObjectProperties.filter((property) => {
      return property.name.toLowerCase().includes(suffix);
    });

    return this.setAutocomplete(autocompletes);
  }

  evaluate () {
    const { input } = this;

    this.logInput(input);
    this.updateInput('');

    setTimeout(() => {
      const { result, error } = evaluate(input);
      let value = error || result;
      const type = error
        ? 'error'
        : 'result';

      if (typeof value === 'string') {
        value = `"${value}"`;
      }

      if (typeof value === 'object' && typeof value.then === 'function') {
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
}

function getAllProperties (object) {
  const propertyNames = {};

  do {
    const prototypeName = object && object.constructor && object.constructor.name || '';

    Object.getOwnPropertyNames(object)
      .sort()
      .forEach((name) => {
        if (Object.prototype.hasOwnProperty.call(propertyNames, name)) {
          return;
        }

        propertyNames[name] = { name, prototypeName };
      });

    object = Object.getPrototypeOf(object);
  } while (object);

  return Object.values(propertyNames);
}

function evaluate (input) {
  try {
    const result = eval(input);

    return { result };
  } catch (err) {
    try {
      const result = eval(`(function () {
        var x = ${input};
        return x;
      })()`);

      return { result };
    } catch (error) {
      return { error };
    }
  }
}
