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

import MethodDecodingStore from '~/ui/MethodDecoding/methodDecodingStore';

const LOGGER_ENABLED = process.env.NODE_ENV !== 'production';

let logger;

if (LOGGER_ENABLED) {
  class Logger {
    _logs = {};
    _id = 0;

    log ({ method, params }) {
      const logId = this._id++;

      this._logs[logId] = { method, params, date: Date.now() };
      return logId;
    }

    set (logId, data) {
      this._logs[logId] = {
        ...this._logs[logId],
        ...data
      };
    }

    static sorter (logA, logB) {
      return logA.date - logB.date;
    }

    get calls () {
      const calls = this.methods['eth_call'] || [];
      const decoding = MethodDecodingStore.get(window.secureApi);
      const contracts = {};

      const progress = Math.round(calls.length / 20);

      return calls
        .reduce((promise, call, index) => {
          const { data, to } = call.params[0];

          contracts[to] = contracts[to] || [];

          return promise
            .then(() => decoding.lookup(null, { data, to }))
            .then((lookup) => {
              if (!lookup.name) {
                contracts[to].push(data);
                return;
              }

              const inputs = lookup.inputs.map((input) => {
                if (/bytes/.test(input.type)) {
                  return '0x' + input.value.map((v) => v.toString(16).padStart(2, 0)).join('');
                }

                return input.value;
              });

              const called = `${lookup.name}(${inputs.join(', ')})`;

              contracts[to].push(called);

              if (index % progress === 0) {
                console.warn(`progress: ${Math.round(100 * index / calls.length)}%`);
              }
            });
        }, Promise.resolve())
        .then(() => {
          return Object.keys(contracts)
            .map((address) => {
              const count = contracts[address].length;

              return {
                count,
                calls: contracts[address],
                to: address
              };
            })
            .sort((cA, cB) => cB.count - cA.count);
        });
    }

    get logs () {
      return Object.values(this._logs).sort(Logger.sorter);
    }

    get methods () {
      return this.logs.reduce((methods, log) => {
        methods[log.method] = methods[log.method] || [];
        methods[log.method].push(log);
        return methods;
      }, {});
    }

    get stats () {
      const logs = this.logs;
      const methods = this.methods;

      const start = logs[0].date;
      const end = logs[logs.length - 1].date;

      // Duration in seconds
      const duration = (end - start) / 1000;
      const speed = logs.length / duration;

      const sortedMethods = Object.keys(methods)
        .map((method) => {
          const methodLogs = methods[method].sort(Logger.sorter);
          const methodSpeed = methodLogs.length / duration;

          return {
            speed: methodSpeed,
            count: methodLogs.length,
            logs: methodLogs,
            method
          };
        })
        .sort((mA, mB) => mB.count - mA.count);

      return {
        methods: sortedMethods,
        speed
      };
    }
  }

  logger = new Logger();

  if (window) {
    window._logger = logger;
  }
}

export default logger;
