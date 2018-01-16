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

// Allow a web worker in the browser, with a fallback for Node.js
const hasWebWorkers = typeof Worker !== 'undefined';
const KeyWorker = hasWebWorkers
  ? require('worker-loader!./worker') // eslint-disable-line import/no-webpack-loader-syntax
  : require('./worker').KeyWorker;

class WorkerContainer {
  constructor () {
    this.busy = false;
    this._worker = new KeyWorker();
  }

  action (action, payload) {
    if (this.busy) {
      throw new Error('Cannot issue an action on a busy worker!');
    }

    this.busy = true;

    return new Promise((resolve, reject) => {
      this._worker.postMessage({ action, payload });
      this._worker.onmessage = ({ data }) => {
        const [err, result] = data;

        this.busy = false;

        if (err) {
          // `err` ought to be a String
          reject(new Error(err));
        } else {
          resolve(result);
        }
      };
    });
  }
}

class WorkerPool {
  constructor () {
    this.pool = [
      new WorkerContainer(),
      new WorkerContainer()
    ];

    this.queue = [];
  }

  _getContainer () {
    return this.pool.find((container) => !container.busy);
  }

  action (action, payload) {
    let container = this.pool.find((container) => !container.busy);

    let promise;

    // const start = Date.now();

    if (container) {
      promise = container.action(action, payload);
    } else {
      promise = new Promise((resolve, reject) => {
        this.queue.push([action, payload, resolve]);
      });
    }

    return promise
      .catch((err) => {
        this.processQueue();

        throw err;
      })
      .then((result) => {
        this.processQueue();

        // console.log('Work done in ', Date.now() - start);

        return result;
      });
  }

  processQueue () {
    let container = this._getContainer();

    while (container && this.queue.length > 0) {
      const [action, payload, resolve] = this.queue.shift();

      resolve(container.action(action, payload));
      container = this._getContainer();
    }
  }
}

module.exports = new WorkerPool();
