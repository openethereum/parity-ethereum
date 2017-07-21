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

import subscribeToEvents from '../util/subscribe-to-events';

export const checkIfVerified = (contract, account) => {
  return contract.instance.certified.call({}, [account]);
};

export const findLastRequested = (contract, account) => {
  let subId = null;
  let resolved = false;

  return new Promise((resolve, reject) => {
    contract
      .subscribe('Requested', {
        fromBlock: 0,
        toBlock: 'pending',
        limit: 1,
        topics: [account]
      }, (err, logs) => {
        if (err) {
          return reject(err);
        }

        resolve(logs[0] || null);
        resolved = true;

        if (subId) {
          contract.unsubscribe(subId);
        }
      })
      .then((_subId) => {
        subId = _subId;

        if (resolved) {
          contract.unsubscribe(subId);
        }
      });
  });
};

const blockNumber = (api) => {
  return new Promise((resolve, reject) => {
    api.subscribe('eth_blockNumber', (err, block) => {
      if (err) {
        return reject(err);
      }
      resolve(block);
    })
    .then((subscription) => {
      api.unsubscribe(subscription);
    })
    .catch(reject);
  });
};

export const awaitPuzzle = (api, contract, account) => {
  return blockNumber(api)
    .then((block) => {
      return new Promise((resolve, reject) => {
        const subscription = subscribeToEvents(contract, ['Puzzled'], {
          from: block.toNumber(),
          filter: (log) => log.params.who.value === account
        });

        subscription.once('error', reject);
        subscription.once('log', subscription.unsubscribe);
        subscription.once('log', resolve);
        subscription.once('timeout', () => {
          reject(new Error('Timed out waiting for the puzzle.'));
        });
      });
    });
};
