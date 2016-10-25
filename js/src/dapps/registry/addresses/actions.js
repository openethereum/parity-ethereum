// Copyright 2015, 2016 Ethcore (UK) Ltd.
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

import { api } from '../parity';

export const set = (addresses) => ({ type: 'addresses set', addresses });

export const fetch = () => (dispatch) => {
  return Promise
    .all([
      api.eth.accounts(),
      null // api.personal.accountsInfo()
    ])
    .then(([ accounts, data ]) => {
      const addresses = accounts.map((address) => {
        return { address, isAccount: true };
      });
      dispatch(set(addresses));
    })
    .catch((error) => {
      console.error('could not fetch addresses', error);
    });
};
