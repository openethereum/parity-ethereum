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

import { sha3, toWei } from '../parity.js';

const alreadyQueued = (queue, action, name) =>
  !!queue.find((entry) => entry.action === action && entry.name === name)

export const reserveStart = (name) => ({ type: 'names reserve start', name });

export const reserveSuccess = (name) => ({ type: 'names reserve success', name });

export const reserveFail = (name) => ({ type: 'names reserve fail', name });

export const reserve = (name) => (dispatch, getState) => {
  const state = getState();
  const account = state.accounts.selected;
  const contract = state.contract;
  if (!contract || !account) return;
  if (alreadyQueued(state.names.queue, 'reserve', name)) return;
  const reserve = contract.functions.find((f) => f.name === 'reserve');

  name = name.toLowerCase();
  const options = {
    from: account.address,
    value: toWei(1).toString()
  };
  const values = [ sha3(name) ];

  dispatch(reserveStart(name));
  reserve.estimateGas(options, values)
    .then((gas) => {
      options.gas = gas.mul(1.2).toFixed(0);
      return reserve.postTransaction(options, values);
    })
    .then((data) => {
      dispatch(reserveSuccess(name));
    }).catch((err) => {
      console.error(`could not reserve ${name}`);
      if (err) console.error(err.stack);
      dispatch(reserveFail(name));
    });
};

export const dropStart = (name) => ({ type: 'names drop start', name });

export const dropSuccess = (name) => ({ type: 'names drop success', name });

export const dropFail = (name) => ({ type: 'names drop fail', name });

export const drop = (name) => (dispatch, getState) => {
  const state = getState();
  const account = state.accounts.selected;
  const contract = state.contract;
  if (!contract || !account) return;
  if (alreadyQueued(state.names.queue, 'drop', name)) return;
  const drop = contract.functions.find((f) => f.name === 'drop');

  name = name.toLowerCase();
  const options = { from: account.address };
  const values = [ sha3(name) ];

  dispatch(dropStart(name));
  drop.estimateGas(options, values)
    .then((gas) => {
      options.gas = gas.mul(1.2).toFixed(0);
      return drop.postTransaction(options, values);
    })
    .then((data) => {
      dispatch(dropSuccess(name));
    }).catch((err) => {
      console.error(`could not drop ${name}`);
      if (err) console.error(err.stack);
      dispatch(reserveFail(name));
    });
};

export const transferStart = (name) => ({ type: 'names transfer start', name });

export const transferSuccess = (name) => ({ type: 'names transfer success', name });

export const transferFail = (name) => ({ type: 'names transfer fail', name });

export const transfer = (name, receiver) => (dispatch, getState) => {
  const state = getState();
  const account = state.accounts.selected;
  const contract = state.contract;
  if (!contract || !account) return;
  if (alreadyQueued(state.names.queue, 'transfer', name)) return;
  const transfer = contract.functions.find((f) => f.name === 'transfer');

  name = name.toLowerCase();
  const options = { from: account.address };
  const values = [ sha3(name), receiver ];

  dispatch(transferStart(name));
  transfer.estimateGas(options, values)
    .then((gas) => {
      options.gas = gas.mul(1.2).toFixed(0);
      return transfer.postTransaction(options, values);
    })
    .then((data) => {
      dispatch(transferSuccess(name));
    }).catch((err) => {
      console.error(`could not transfer ${name}`);
      if (err) console.error(err.stack);
      dispatch(reserveFail(name));
    });
};
