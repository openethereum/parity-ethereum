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

let instance = null;

export default class PlaygroundStore {
  @observable components = [];

  static get () {
    if (!instance) {
      instance = new PlaygroundStore();
    }

    return instance;
  }

  static register (component) {
    PlaygroundStore.get().add(component);
  }

  @action
  add (component) {
    const name = component.type.displayName || component.type.name;
    const hasComponent = this.components.find((c) => {
      const cName = c.type.displayName || c.type.name;

      return name && cName && cName === name;
    });

    if (hasComponent) {
      return;
    }

    this.components.push(component);
  }
}
