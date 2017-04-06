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
const KeyWorker = hasWebWorkers ? require('worker-loader!./worker')
                                : require('./worker').KeyWorker;

class WorkerContainer {
  busy = false;
  _worker = new KeyWorker();

  action (action, payload) {
    if (this.busy) {
      throw new Error('Cannot issue an action on a busy worker!');
    }

    this.busy = true;

    return new Promise((resolve, reject) => {
      this._worker.postMessage({ action, payload });
      this._worker.onmessage = ({ data }) => {
        this.busy = false;
        resolve(data);
      };
    });
  }
}

class WorkerPool {
  pool = [];

  getWorker () {
    let container = this.pool.find((container) => !container.busy);

    if (container) {
      return container;
    }

    container = new WorkerContainer();

    this.pool.push(container);

    return container;
  }
}

export default new WorkerPool();
