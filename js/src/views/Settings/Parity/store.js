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

import LogLevel from 'loglevel';
import { action, observable } from 'mobx';

import { LOG_KEYS } from '~/config';

const DEFAULT_MODE = 'active';
const DEFAULT_CHAIN = 'foundation';
const LOGLEVEL_OPTIONS = Object
  .keys(LogLevel.levels)
  .map((name) => {
    return {
      name,
      value: LogLevel.levels[name]
    };
  });

export default class Store {
  @observable logLevels = {};
  @observable mode = DEFAULT_MODE;
  @observable chain = DEFAULT_CHAIN;

  constructor (api) {
    this._api = api;

    this.loadLogLevels();
  }

  @action setLogLevels = (logLevels) => {
    this.logLevels = { ...logLevels };
  }

  @action setLogLevelsSelect = (logLevelsSelect) => {
    this.logLevelsSelect = logLevelsSelect;
  }

  @action setMode = (mode) => {
    this.mode = mode;
  }

  @action setChain = (chain) => {
    this.chain = chain;
  }

  changeMode (mode) {
    return this._api.parity
      .setMode(mode)
      .then((result) => {
        if (result) {
          this.setMode(mode);
        }
      })
      .catch((error) => {
        console.warn('changeMode', error);
      });
  }

  changeChain (chain) {
    return this._api.parity
      .setChain(chain)
      .then((result) => {
        if (result) {
          this.setChain(chain);
        }
      })
      .catch((error) => {
        console.warn('changeChain', error);
      });
  }

  loadLogLevels () {
    this.setLogLevels(
      Object
        .keys(LOG_KEYS)
        .reduce((state, logKey) => {
          const log = LOG_KEYS[logKey];
          const logger = LogLevel.getLogger(log.key);
          const level = logger.getLevel();

          state[logKey] = {
            level,
            log
          };

          return state;
        }, this.logLevels)
    );
  }

  updateLoggerLevel (key, level) {
    LogLevel.getLogger(key).setLevel(level);
    this.loadLogLevels();
  }

  loadMode () {
    return this._api.parity
      .mode()
      .then((mode) => {
        this.setMode(mode);
      })
      .catch((error) => {
        console.warn('loadMode', error);
      });
  }

  loadChain () {
    return this._api.parity
      .chain()
      .then((chain) => {
        this.setChain(chain);
      })
      .catch((error) => {
        console.warn('loadChain', error);
      });
  }
}

export {
  LOGLEVEL_OPTIONS
};
