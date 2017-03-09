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

import { api, sha3 } from '../parity.js';
import { getOwner } from '../util/registry';

export const clear = () => ({ type: 'lookup clear' });

export const lookupStart = (name, key) => ({ type: 'lookup start', name, key });
export const reverseLookupStart = (address) => ({ type: 'reverseLookup start', address });
export const ownerLookupStart = (name) => ({ type: 'ownerLookup start', name });

export const success = (action, result) => ({ type: `${action} success`, result: result });

export const fail = (action) => ({ type: `${action} error` });

export const lookup = (name, key) => (dispatch, getState) => {
  const { contract } = getState();

  if (!contract) {
    return;
  }

  const method = key === 'A'
    ? contract.instance.getAddress
    : contract.instance.getData;

  name = name.toLowerCase();
  dispatch(lookupStart(name, key));

  method.call({}, [ sha3.text(name), key ])
    .then((result) => {
      if (key !== 'A') {
        result = api.util.bytesToHex(result);
      }

      dispatch(success('lookup', result));
    })
    .catch((err) => {
      console.error(`could not lookup ${key} for ${name}`);
      if (err) {
        console.error(err.stack);
      }
      dispatch(fail('lookup'));
    });
};

export const reverseLookup = (lookupAddress) => (dispatch, getState) => {
  const { contract } = getState();

  if (!contract) {
    return;
  }

  dispatch(reverseLookupStart(lookupAddress));

  contract.instance
    .reverse
    .call({}, [ lookupAddress ])
    .then((address) => {
      dispatch(success('reverseLookup', address));
    })
    .catch((err) => {
      console.error(`could not lookup reverse for ${lookupAddress}`);
      if (err) {
        console.error(err.stack);
      }
      dispatch(fail('reverseLookup'));
    });
};

export const ownerLookup = (name) => (dispatch, getState) => {
  const { contract } = getState();

  if (!contract) {
    return;
  }

  name = name.toLowerCase();
  dispatch(ownerLookupStart(name));

  return getOwner(contract, name)
    .then((owner) => {
      dispatch(success('ownerLookup', owner));
    })
    .catch((err) => {
      console.error(`could not lookup owner for ${name}`);

      if (err) {
        console.error(err.stack);
      }

      dispatch(fail('ownerLookup'));
    });
};
