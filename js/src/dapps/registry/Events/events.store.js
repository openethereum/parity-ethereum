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

import { action, map, observable } from 'mobx';

import ApplicationStore from '../Application/application.store';

let instance;

export default class EventsStore {
  @observable events = [];
  @observable subscriptions = map();

  applicationStore = ApplicationStore.get();

  static get () {
    if (!instance) {
      instance = new EventsStore();
    }

    return instance;
  }

  @action
  addEvents (_events) {
    const ids = this.events.map((event) => event.id);
    const events = _events
      .filter((event) => event)
      .filter((event) => !ids.includes(event.id));

    this.events = this.events
      .concat(events)
      .sort((eventA, eventB) => {
        const blockCompare = eventA.block.minus(eventB.block);

        if (!blockCompare.eq(0)) {
          return blockCompare.gt(0)
            ? -1
            : 1;
        }

        return eventA.index.minus(eventB.index).gt(0)
          ? -1
          : 1;
      });
  }

  @action
  removeEvents (type) {
    this.events = this.events.filter((event) => event.type !== type);
  }

  @action
  removeSubscription (name) {
    this.subscriptions.delete(name);
  }

  @action
  setSubscription (name, value) {
    this.subscriptions.set(name, value);
  }

  subscribe (name, from = 0, to = 'pending') {
    if (Array.isArray(name)) {
      const promises = name.map((key) => this.subscribe(key));

      return Promise.all(promises);
    }

    if (this.subscriptions.has(name)) {
      return Promise.resolve();
    }

    const { api, contract } = this.applicationStore;
    const options = { fromBlock: from, toBlock: to, limit: 50 };

    return contract
      .subscribe(name, options, (error, events) => {
        if (error) {
          console.error(`error receiving events for ${name}`, error);
          return;
        }

        const promises = events.map((event) => {
          return Promise.all([
            api.parity.getBlockHeaderByNumber(event.blockNumber),
            api.eth.getTransactionByHash(event.transactionHash)
          ])
          .then(([block, transaction]) => {
            const data = {
              type: name,
              id: `${event.transactionHash}_${event.logIndex}`,
              state: event.type,
              block: event.blockNumber,
              index: event.logIndex,
              transactionHash: event.transactionHash,
              from: transaction.from,
              to: transaction.to,
              parameters: event.params,
              timestamp: block.timestamp
            };

            return data;
          })
          .catch((err) => {
            console.error(`could not fetch block ${event.blockNumber}.`, err);
            return null;
          });
        });

        Promise.all(promises)
          .then((eventsData) => {
            this.addEvents(eventsData);
          });
      })
      .then((subscriptionId) => {
        this.setSubscription(name, subscriptionId);
      })
      .catch((error) => {
        console.error('event subscription failed', error);
      });
  }

  unsubscribe (name) {
    if (Array.isArray(name)) {
      const promises = name.map((key) => this.unsubscribe(key));

      return Promise.all(promises);
    }

    const { contract } = this.applicationStore;

    this.removeEvents(name);

    if (!this.subscriptions.has(name)) {
      return Promise.resolve();
    }

    return contract
      .unsubscribe(this.subscriptions.get(name))
      .catch((error) => {
        console.error('event unsubscribe failed', error);
      })
      .then(() => {
        this.removeSubscription(name);
      });
  }
}
