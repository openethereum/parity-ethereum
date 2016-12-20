// Copyright 2015, 2016 Parity Technologies (UK) Ltd.
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

let workerRegistration;

// Setup the Service Worker
if ('serviceWorker' in navigator) {
  workerRegistration = runtime
    .register()
    .then(() => navigator.serviceWorker.ready)
    .then((registration) => {
      const _worker = registration.active;
      _worker.controller = registration.active;
      const worker = new PromiseWorker(_worker);

      return worker;
    });
} else {
  workerRegistration = Promise.reject('Service Worker is not available in your browser.');
}

export function setWorker (worker) {
  return {
    type: 'setWorker',
    worker
  };
}

export function setError (error) {
  return {
    type: 'setError',
    error
  };
}

export function setupWorker () {
  return (dispatch, getState) => {
    const state = getState();

    if (state.compiler.worker) {
      return;
    }

    workerRegistration
      .then((worker) => {
        dispatch(setWorker(worker));
      })
      .catch((error) => {
        console.error('sw', error);
        dispatch(setWorker(null));
      });
  };
}
