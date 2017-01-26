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

import PromiseWorker from 'promise-worker';
import runtime from 'serviceworker-webpack-plugin/lib/runtime';

import { setWorker } from './workerActions';

function getWorker () {
  // Setup the Service Worker
  if ('serviceWorker' in navigator) {
    return runtime
      .register()
      .then(() => navigator.serviceWorker.ready)
      .then((registration) => {
        const worker = registration.active;

        worker.controller = registration.active;

        return new PromiseWorker(worker);
      });
  }

  return Promise.reject('Service Worker is not available in your browser.');
}

export const setupWorker = (store) => {
  const { dispatch, getState } = store;

  const state = getState();
  const stateWorker = state.worker.worker;

  if (stateWorker !== undefined && !(stateWorker && stateWorker._worker.state === 'redundant')) {
    return;
  }

  getWorker()
    .then((worker) => {
      if (worker) {
        worker._worker.addEventListener('statechange', (event) => {
          console.warn('worker state changed to', worker._worker.state);

          // Re-install the new Worker
          if (worker._worker.state === 'redundant') {
            setupWorker(store);
          }
        });
      }

      dispatch(setWorker(worker));
    })
    .catch((error) => {
      console.error('sw', error);
      dispatch(setWorker(null));
    });
};
