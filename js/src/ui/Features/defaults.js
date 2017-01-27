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

const MODES = {
  DEVELOPMENT: 1000, // only in dev mode, disabled by default, can be toggled
  TESTING: 1011, // feature is available in dev mode
  PRODUCTION: 1022 // feature is available
};

const FEATURES = {
  LANGUAGE: 'LANGUAGE',
  LOGLEVELS: 'LOGLEVELS'
};

const DEFAULTS = {
  [FEATURES.LANGUAGE]: {
    mode: MODES.TESTING,
    name: 'Language Selection',
    description: 'Allows changing the default interface language'
  },
  [FEATURES.LOGLEVELS]: {
    mode: MODES.TESTING,
    name: 'Logging Level Selection',
    description: 'Allows changing of the log levels for various components'
  }
};

if (process.env.NODE_ENV === 'test') {
  Object
    .keys(MODES)
    .forEach((mode) => {
      const key = `.${mode}`;

      FEATURES[key] = key;
      DEFAULTS[key] = {
        mode: MODES[mode],
        name: key,
        description: key
      };
    });
}

export default DEFAULTS;

export {
  FEATURES,
  MODES
};
