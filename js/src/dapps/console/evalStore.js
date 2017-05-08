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

import { action, observable } from 'mobx';

let instance;

export default class EvalStore {
  @observable input = '';
  @observable logs = [];

  static get () {
    if (!instance) {
      instance = new EvalStore();
    }

    return instance;
  }

  @action
  updateInput (nextValue = '') {
    this.input = nextValue;
  }

  @action
  log ({ type, value, error }) {
    this.logs.push({
      type, value,
      timestamp: Date.now()
    });
  }

  evaluate () {
    const { input } = this;

    this.log({ type: 'input', value: input });
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
