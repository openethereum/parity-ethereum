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

const contractCall = (fn, computeValues, computeOptions, computeAction) => (...args) =>
  (dispatch, getState) => {
    const name = args[0]

    const state = getState();
    const account = state.accounts.selected;
    const contract = state.contract;
    if (!contract || !account) return;
    if (alreadyQueued(state.names.queue, fn.name, name)) return;
    fn = contract.functions.find((f) => f.name === fn);

    const options = computeOptions(account)
    const values = computeValues(...args)

    dispatch(computeAction('start', ...args));
    fn.estimateGas(options, values)
      .then((gas) => {
        options.gas = gas.mul(1.2).toFixed(0);
        return fn.postTransaction(options, values);
      })
      .then(() => {
        dispatch(computeAction('success', ...args));
      }).catch((err) => {
        console.error(`could not ${fn.name} ${name}`);
        if (err) console.error(err.stack);
        dispatch(computeAction('fail', ...args));
      });
  }

export const reserve = contractCall(
  'reserve',
  // compute values
  (name) => [ sha3(name) ],
  // compute options
  (account) => ({ from: account.address, value: toWei(1).toString() }),
  // compute action
  (status, name) => ({ type: 'names reserve ' + status, name })
)

export const drop = contractCall(
  'drop',
  // compute values
  (name) => [ sha3(name) ],
  // compute options
  (account) => ({ from: account.address }),
  // compute action
  (status, name) => ({ type: 'names drop ' + status, name })
)

export const transfer = contractCall(
  'transfer',
  // compute values
  (name, receiver) => [ sha3(name), receiver ],
  // compute options
  (account) => ({ from: account.address }),
  // compute action
  (status, name) => ({ type: 'names transfer ' + status, name })
)
