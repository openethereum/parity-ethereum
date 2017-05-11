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

import { evaluate } from '../utils';

let instance;

export default class AutocompleteStore {
  @observable values = [];
  @observable position = {};
  @observable show = false;
  @observable selected = null;

  elements = {};
  inputNode = null;
  lastObject = null;
  lastObjectPropertyNames = [];

  static get () {
    if (!instance) {
      instance = new AutocompleteStore();
    }

    return instance;
  }

  get hasSelected () {
    return this.selected !== null;
  }

  clearCache () {
    this.lastObject = null;
    this.lastObjectPropertyNames = null;
  }

  @action
  focus (offset = 1) {
    if (this.values.length === 0) {
      this.selected = null;
      return;
    }

    this.selected = this.selected === null
      ? (
        offset === 1
          ? 0
          : this.values.length - 1
      )
      : (this.values.length + this.selected + offset) % (this.values.length);

    if (this.isVisible(this.selected)) {
      return;
    }

    const element = this.elements[this.selected];

    if (!element) {
      return;
    }

    element.scrollIntoView(offset === -1);
  }

  focusOnInput () {
    if (!this.inputNode) {
      return;
    }

    this.inputNode.focus();
  }

  @action
  hide () {
    this.show = false;
    this.selected = null;
  }

  isVisible (index) {
    const element = this.elements[index];

    if (!element) {
      return false;
    }

    const eBoundings = element.getBoundingClientRect();
    const pBoundings = element.parentElement.getBoundingClientRect();

    if (eBoundings.top < pBoundings.top || eBoundings.bottom > pBoundings.bottom) {
      return false;
    }

    return true;
  }

  select (inputStore, _index = this.selected) {
    const index = _index === null
      ? 0
      : _index;

    if (!this.values[index]) {
      console.warn(`autocomplete::select has been called on AutocompleteStore with wrong value ${index}`);
      return;
    }

    const { name } = this.values[index];
    const { input } = inputStore;
    const objects = input.split('.');

    objects[objects.length - 1] = name;
    const nextInput = objects.join('.');

    this.hide();
    this.focusOnInput();
    return inputStore.updateInput(nextInput, false);
  }

  setElement (index, element) {
    this.elements[index] = element;
  }

  setInputNode (node) {
    this.inputNode = node;
  }

  @action
  setPosition () {
    if (!this.inputNode) {
      return;
    }

    const inputBoundings = this.inputNode.getBoundingClientRect();
    const bodyBoundings = document.body.getBoundingClientRect();

    // display on bottom of input
    if (inputBoundings.top < bodyBoundings.height / 2) {
      const nextPosition = {
        top: 20
      };

      this.position = nextPosition;
      return;
    }

    // display on top of input
    const nextPosition = {
      bottom: inputBoundings.height
    };

    this.position = nextPosition;
    return;
  }

  @action
  setValues (values) {
    this.values = values;
    this.selected = null;
    const show = values.length > 0;

    // Reveal autocomplete
    if (!this.show && show) {
      this.setPosition();
    }

    this.show = show;
  }

  update (input) {
    if (input.length === 0) {
      return this.setValues([]);
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

    return this.setValues(autocompletes);
  }
}

function getAllProperties (object) {
  const propertyNames = {};

  while (object) {
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
  }

  return Object.values(propertyNames);
}
