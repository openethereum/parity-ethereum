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

import { action, computed, map, observable } from 'mobx';

import ApplicationStore from '../Application/application.store';

let instance;

export default class EventsStore {
  @observable allEvents = [];
  @observable shown = map();

  applicationStore = ApplicationStore.get();
  subscriptionId = null;

  constructor () {
    this.subscribe();
  }

  static get () {
    if (!instance) {
      instance = new EventsStore();
    }

    return instance;
  }

  @computed
  get events () {
    const types = this.shown.values().reduce((types, current) => {
      return types.concat(current.slice());
    }, []);

    return this.allEvents.filter((event) => types.includes(event.type));
  }

  @action
  addEvents (_events) {
    const allEvents = this.allEvents.slice();
    const ids = allEvents.map((event) => event.id);
    const events = _events
      .filter((event) => event);

    events.forEach((event) => {
      const index = ids.indexOf(event.id);

      if (index === -1) {
        return allEvents.push(event);
      }

      allEvents[index] = event;
    });

    this.allEvents = allEvents.slice()
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

  subscribe (from = 0, to = 'pending') {
    const { api, contract } = this.applicationStore;
    const options = { fromBlock: from, toBlock: to, limit: 50 };

    return contract
      .subscribe(null, options, (error, events) => {
        if (error) {
          console.error(`error receiving events`, error);
          return;
        }

        const promises = events.map((event) => {
          return Promise.all([
            api.parity.getBlockHeaderByNumber(event.blockNumber),
            api.eth.getTransactionByHash(event.transactionHash)
          ])
          .then(([block, transaction]) => {
            const data = {
              type: event.event,
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
        this.subscriptionId = subscriptionId;
      })
      .catch((error) => {
        console.error('event subscription failed', error);
      });
  }

  @action
  toggle (key, toggled) {
    if (!toggled) {
      return this.shown.delete(key);
    }

    if (key === 'reservations') {
      return this.shown.set(key, [ 'Reserved', 'Transferred', 'Dropped' ]);
    }

    if (key === 'metadata') {
      return this.shown.set(key, [ 'DataChanged' ]);
    }

    if (key === 'reverses') {
      return this.shown.set(key, [ 'ReverseConfirmed', 'ReverseRemoved', 'ReverseProposed' ]);
    }
  }

  unsubscribe () {
    const { contract } = this.applicationStore;

    if (!this.subscriptionId) {
      return Promise.resolve();
    }

    return contract
      .unsubscribe(this.subscriptionId)
      .catch((error) => {
        console.error('event unsubscribe failed', error);
      });
  }
}
