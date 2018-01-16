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

let instance;

export const DISPLAY_ACCOUNTS = 'accounts';
export const DISPLAY_SIGNER = 'signer';

export default class Store {
  @observable isOpen = false;
  @observable displayType = DISPLAY_SIGNER;

  @action setOpen = (isOpen, displayType) => {
    this.isOpen = isOpen;
    this.displayType = displayType;
  }

  @action toggleOpen = (displayType) => {
    this.setOpen(!this.isOpen, displayType);
  }

  @action toggleOpenAccounts = () => {
    this.toggleOpen(DISPLAY_ACCOUNTS);
  }

  @action toggleOpenSigner = () => {
    this.toggleOpen(DISPLAY_SIGNER);
  }

  static get () {
    if (!instance) {
      instance = new Store();
    }

    return instance;
  }
}
