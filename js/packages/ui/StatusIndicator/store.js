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

import { action, computed, observable } from 'mobx';

const STATUS_OK = 'ok';
const STATUS_WARN = 'needsAttention';
const STATUS_BAD = 'bad';
const EMPTY_OVERALL = { message: [], status: STATUS_BAD };

export default class Store {
  @observable _health = null;

  constructor (api) {
    this._api = api;

    setInterval(this.fetchHealth, 2500);
  }

  @computed get health () {
    return this._health
      ? this._health
      : {};
  }

  @computed get overall () {
    return this._health
      ? this._health.overall
      : EMPTY_OVERALL;
  }

  fetchHealth = () => {
    // Support Parity-Extension.
    const uiUrl = this._api.transport.uiUrlWithProtocol || '';

    return fetch(`${uiUrl}/api/health`)
      .then((response) => {
        if (!response.ok) {
          return null;
        }

        return response.json();
      })
      .catch(() => {
        return null;
      })
      .then(this.setHealth);
  }

  _overallStatus = (health) => {
    const all = [health.peers, health.sync, health.time].filter(x => x);
    const statuses = all.map(x => x.status);
    const bad = statuses.find(x => x === STATUS_BAD);
    const needsAttention = statuses.find(x => x === STATUS_WARN);
    const message = all.map(x => x.message).filter(x => x);

    if (all.length) {
      return {
        status: bad || needsAttention || STATUS_OK,
        message
      };
    }

    return {
      status: STATUS_BAD,
      message: ['Unable to fetch node health.']
    };
  }

  @action setHealth = (health) => {
    if (!health) {
      this._health = null;
      return;
    }

    health.peers = health.peers || {};
    health.sync = health.sync || {};
    health.time = health.time || {};
    health.overall = this._overallStatus(health);

    this._health = health;
  }

  static instance = null;

  static get (api) {
    if (!Store.instance) {
      Store.instance = new Store(api);
    }

    return Store.instance;
  }
}
