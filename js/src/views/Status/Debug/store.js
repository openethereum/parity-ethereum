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

import { action, observable, transaction } from 'mobx';

const LOG_DATE_REGEX = /^(\d{4}.\d{2}.\d{2}.\d{2}.\d{2}.\d{2})(.*)$/i;
const MAX_LOGS = 25;

export default class DebugStore {
  @observable logs = [];
  @observable logsLevels = null;
  @observable logsEnabled = false;
  @observable reversed = false;

  api = null;
  _lastLogAdded = null;
  _timeoutId = null;

  constructor (api) {
    this.api = api;
  }

  @action clearLogs () {
    this.logs = [];
  }

  @action setLogs (logs, logsLevels) {
    let newLogs = [];

    if (this._lastLogAdded) {
      const sliceIndex = logs.findIndex((log) => log === this._lastLogAdded);

      newLogs = logs.slice(0, sliceIndex);
    } else {
      newLogs = logs.slice();
    }

    this._lastLogAdded = logs[0];

    const parsedLogs = newLogs
      .map((log) => {
        const logDate = LOG_DATE_REGEX.exec(log);

        if (!logDate) {
          return null;
        }

        return {
          date: new Date(logDate[1]),
          log: logDate[2]
        };
      })
      .filter((log) => log);

    transaction(() => {
      if (!this.reversed) {
        this.logs = [].concat(parsedLogs, this.logs.slice()).slice(0, MAX_LOGS);
      } else {
        parsedLogs.reverse();
        this.logs = [].concat(this.logs.slice(), parsedLogs).slice(-1 * MAX_LOGS);
      }

      this.logsLevels = logsLevels;
    });
  }

  @action toggle () {
    this.logsEnabled = !this.logsEnabled;

    if (this.logsEnabled) {
      this.initPolling();
    } else {
      this.stopPolling();
    }
  }

  @action reverse () {
    transaction(() => {
      this.reversed = !this.reversed;
      this.logs = this.logs.reverse();
    });
  }

  initPolling () {
    this._pollLogs();
  }

  stopPolling () {
    if (this._timeoutId) {
      clearTimeout(this._timeoutId);
    }
  }

  _pollLogs = () => {
    const nextTimeout = (timeout = 1000) => {
      this.stopPolling();
      this._timeoutId = setTimeout(this._pollLogs, timeout);
    };

    return Promise
      .all([
        this.api.parity.devLogs(),
        this.api.parity.devLogsLevels()
      ])
      .then(([ devLogs, devLogsLevels ]) => {
        this.setLogs(devLogs, devLogsLevels);
      })
      .catch((error) => {
        console.error('_pollLogs', error);
      })
      .then(() => {
        return nextTimeout();
      });
  }
}
