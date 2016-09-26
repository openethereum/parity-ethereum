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

export const start = (name) => ({ type: 'register start', name });

export const success = (name) => ({ type: 'register success', name });

export const fail = (name) => ({ type: 'register fail', name });

export const register = (name) => (dispatch, getState) => {
  const state = getState();
  const account = state.accounts.selected;
  const contract = state.contract;
  if (!contract || !account) return;
  if (state.register.posted.includes(name)) return;
  const reserve = contract.functions.find((f) => f.name === 'reserve');

  name = name.toLowerCase();
  const options = {
    from: account.address,
    value: toWei(1).toString()
  };
  const values = [ sha3(name) ];

  dispatch(start(name));
  reserve.estimateGas(options, values)
    .then((gas) => {
      options.gas = gas.mul(1.2).toFixed(0);
      return reserve.postTransaction(options, values);
    })
    .then((data) => {
      dispatch(success(name));
    }).catch((err) => {
      console.error(`could not reserve ${name}`);
      if (err) console.error(err.stack);
      dispatch(fail(name));
    });
};
