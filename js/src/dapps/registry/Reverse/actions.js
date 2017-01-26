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

import { api } from '../parity.js';
import postTx from '../util/post-tx';
import { getOwner } from '../util/registry';

export const clearError = () => ({
  type: 'clearError'
});

export const start = (action, name, address) => ({ type: `reverse ${action} start`, name, address });

export const success = (action) => ({ type: `reverse ${action} success` });

export const fail = (action, error) => ({ type: `reverse ${action} fail`, error });

export const propose = (name, address) => (dispatch, getState) => {
  const state = getState();
  const account = state.accounts.selected;
  const contract = state.contract;

  if (!contract || !account) {
    return;
  }

  name = name.toLowerCase();
  dispatch(start('propose', name, address));

  return getOwner(contract, name)
    .then((owner) => {
      if (owner.toLowerCase() !== account.address.toLowerCase()) {
        throw new Error(`you are not the owner of "${name}"`);
      }

      const { proposeReverse } = contract.instance;

      const options = {
        from: account.address
      };

      const values = [
        name,
        address
      ];

      return postTx(api, proposeReverse, options, values);
    })
    .then((txHash) => {
      dispatch(success('propose'));
    })
    .catch((err) => {
      if (err.type !== 'REQUEST_REJECTED') {
        console.error(`error proposing ${name}`, err);
        return dispatch(fail('propose', err));
      }

      dispatch(fail('propose'));
    });
};

export const confirm = (name) => (dispatch, getState) => {
  const state = getState();
  const account = state.accounts.selected;
  const contract = state.contract;

  if (!contract || !account) {
    return;
  }

  name = name.toLowerCase();
  dispatch(start('confirm', name));

  return getOwner(contract, name)
    .then((owner) => {
      if (owner.toLowerCase() !== account.address.toLowerCase()) {
        throw new Error(`you are not the owner of "${name}"`);
      }

      const { confirmReverse } = contract.instance;

      const options = {
        from: account.address
      };

      const values = [
        name
      ];

      return postTx(api, confirmReverse, options, values);
    })
    .then((txHash) => {
      dispatch(success('confirm'));
    })
    .catch((err) => {
      if (err.type !== 'REQUEST_REJECTED') {
        console.error(`error confirming ${name}`, err);
        return dispatch(fail('confirm', err));
      }

      dispatch(fail('confirm'));
    });
};

