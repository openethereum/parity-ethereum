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

import { sha3, api } from '../parity.js';
import { getOwner, isOwned } from '../util/registry';
import postTx from '../util/post-tx';

export const clearError = () => ({
  type: 'clearError'
});

const alreadyQueued = (queue, action, name) =>
  !!queue.find((entry) => entry.action === action && entry.name === name);

export const reserveStart = (name) => ({ type: 'names reserve start', name });

export const reserveSuccess = (name) => ({ type: 'names reserve success', name });

export const reserveFail = (name, error) => ({ type: 'names reserve fail', name, error });

export const reserve = (name) => (dispatch, getState) => {
  const state = getState();
  const account = state.accounts.selected;
  const contract = state.contract;
  const fee = state.fee;

  if (!contract || !account) {
    return;
  }

  name = name.toLowerCase();

  if (alreadyQueued(state.names.queue, 'reserve', name)) {
    return;
  }

  dispatch(reserveStart(name));

  return isOwned(contract, name)
    .then((owned) => {
      if (owned) {
        throw new Error(`"${name}" has already been reserved`);
      }

      const { reserve } = contract.instance;

      const options = {
        from: account.address,
        value: fee
      };
      const values = [
        sha3.text(name)
      ];

      return postTx(api, reserve, options, values);
    })
    .then((txHash) => {
      dispatch(reserveSuccess(name));
    })
    .catch((err) => {
      if (err.type !== 'REQUEST_REJECTED') {
        console.error(`error rerserving ${name}`, err);
        return dispatch(reserveFail(name, err));
      }

      dispatch(reserveFail(name));
    });
};

export const dropStart = (name) => ({ type: 'names drop start', name });

export const dropSuccess = (name) => ({ type: 'names drop success', name });

export const dropFail = (name, error) => ({ type: 'names drop fail', name, error });

export const drop = (name) => (dispatch, getState) => {
  const state = getState();
  const account = state.accounts.selected;
  const contract = state.contract;

  if (!contract || !account) {
    return;
  }

  name = name.toLowerCase();

  if (alreadyQueued(state.names.queue, 'drop', name)) {
    return;
  }

  dispatch(dropStart(name));

  return getOwner(contract, name)
    .then((owner) => {
      if (owner.toLowerCase() !== account.address.toLowerCase()) {
        throw new Error(`you are not the owner of "${name}"`);
      }

      const { drop } = contract.instance;

      const options = {
        from: account.address
      };

      const values = [
        sha3.text(name)
      ];

      return postTx(api, drop, options, values);
    })
    .then((txhash) => {
      dispatch(dropSuccess(name));
    })
    .catch((err) => {
      if (err.type !== 'REQUEST_REJECTED') {
        console.error(`error dropping ${name}`, err);
        return dispatch(dropFail(name, err));
      }

      dispatch(dropFail(name));
    });
};
