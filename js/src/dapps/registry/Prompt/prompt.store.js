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

import { action, observable, transaction } from 'mobx';

let instance;

export default class PromptStore {
  @observable defaultValue = null;
  @observable placeholder = null;
  @observable show = false;
  @observable showInput = false;
  @observable title = null;

  input = null;
  resolve = null;
  reject= null;

  static get () {
    if (!instance) {
      instance = new PromptStore();
    }

    return instance;
  }

  ask ({ defaultValue, title, placeholder, showInput = false }) {
    if (this.resolve || this.reject) {
      throw Error('already showing a prompt');
    }

    return new Promise((resolve, reject) => {
      this.resolve = resolve;
      this.reject = reject;

      this.setParameters({ defaultValue, title, placeholder, showInput });
    });
  }

  close () {
    this.reject();
    this.clean();
  }

  @action
  clean () {
    this.show = false;
    this.reject = null;
    this.resolve = null;
  }

  @action
  setParameters ({ defaultValue, title, placeholder, showInput }) {
    transaction(() => {
      this.show = true;

      this.defaultValue = defaultValue;
      this.placeholder = placeholder;
      this.title = title;
      this.showInput = showInput;
      this.input = defaultValue || '';
    });
  }

  submit () {
    this.resolve(this.input);
    this.clean();
  }

  updateInput (value) {
    this.input = value;
  }
}
