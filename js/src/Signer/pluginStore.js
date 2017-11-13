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

export default class PluginStore {
  @observable plugins = [];

  @action addComponent (Component, isHandler, isFallback) {
    if (!Component || (typeof isHandler !== 'function')) {
      throw new Error(`Unable to attach Signer plugin, 'React Component' or 'isHandler' function is not defined`);
    }

    this.plugins.push({
      Component,
      isHandler,
      isFallback
    });

    return true;
  }

  findPayloadAccount (payload, accounts) {
    if (payload.decrypt) {
      return accounts[payload.decrypt.address];
    } else if (payload.sign) {
      return accounts[payload.sign.address];
    } else if (payload.sendTransaction) {
      return accounts[payload.sendTransaction.from];
    } else if (payload.signTransaction) {
      return accounts[payload.signTransaction.from];
    }

    return null;
  }

  findFallback (payload, accounts, account) {
    const plugin = this.plugins.find((p) => {
      try {
        return !!(
          p.isFallback &&
          p.isHandler(payload, accounts, account)
        );
      } catch (error) {
        return false;
      }
    });

    return plugin
      ? plugin.Component
      : null;
  }

  findHandler (payload, accounts) {
    const account = this.findPayloadAccount(payload, accounts);
    const plugin = this.plugins.find((p) => {
      try {
        return !!(
          !p.isFallback &&
          p.isHandler(payload, accounts, account)
        );
      } catch (error) {
        return false;
      }
    });

    return plugin
      ? plugin.Component
      : this.findFallback(payload, accounts, account);
  }

  static get () {
    if (!instance) {
      instance = new PluginStore();
    }

    return instance;
  }
}
