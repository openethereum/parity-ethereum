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

import { api } from '../parity.js';

export const start = (name, from, to) => ({ type: 'events subscribe start', name, from, to });
export const fail = (name) => ({ type: 'events subscribe fail', name });
export const success = (name, subscription) => ({ type: 'events subscribe success', name, subscription });

export const event = (name, event) => ({ type: 'events event', name, event });

export const subscribe = (name, from = 0, to = 'pending') =>
  (dispatch, getState) => {
    const { contract } = getState();

    if (!contract) {
      return;
    }

    const opt = { fromBlock: from, toBlock: to, limit: 50 };

    dispatch(start(name, from, to));

    contract
      .subscribe(name, opt, (error, events) => {
        if (error) {
          console.error(`error receiving events for ${name}`, error);
          return;
        }

        events.forEach((e) => {
          Promise.all([
            api.parity.getBlockHeaderByNumber(e.blockNumber),
            api.eth.getTransactionByHash(e.transactionHash)
          ])
          .then(([block, tx]) => {
            const data = {
              type: name,
              key: '' + e.transactionHash + e.logIndex,
              state: e.type,
              block: e.blockNumber,
              index: e.logIndex,
              transaction: e.transactionHash,
              from: tx.from,
              to: tx.to,
              parameters: e.params,
              timestamp: block.timestamp
            };

            dispatch(event(name, data));
          })
          .catch((err) => {
            console.error(`could not fetch block ${e.blockNumber}.`);
            console.error(err);
          });
        });
      })
      .then((subscriptionId) => {
        dispatch(success(name, subscriptionId));
      })
      .catch((error) => {
        console.error('event subscription failed', error);
        dispatch(fail(name));
      });
  };

export const unsubscribe = (name) =>
  (dispatch, getState) => {
    const state = getState();

    if (!state.contract) {
      return;
    }

    const subscriptions = state.events.subscriptions;

    if (!(name in subscriptions) || subscriptions[name] === null) {
      return;
    }

    state.contract
      .unsubscribe(subscriptions[name])
      .then(() => {
        dispatch({ type: 'events unsubscribe', name });
      })
      .catch((error) => {
        console.error('event unsubscribe failed', error);
      });
  };
