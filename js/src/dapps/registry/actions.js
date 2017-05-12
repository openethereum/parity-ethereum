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

import { api } from './parity.js';
import * as addresses from './addresses/actions.js';
import * as accounts from './Accounts/actions.js';
import * as events from './Events/actions.js';
import * as names from './Names/actions.js';
import * as records from './Records/actions.js';
import * as reverse from './Reverse/actions.js';

export { addresses, accounts, events, names, records, reverse };

export const setNetVersion = (netVersion) => ({ type: 'set netVersion', netVersion });

export const fetchIsTestnet = () => (dispatch) =>
  api.net.version()
    .then((netVersion) => {
      dispatch(setNetVersion(netVersion));
    })
    .catch((err) => {
      console.error('could not check if testnet');
      if (err) {
        console.error(err.stack);
      }
    });
