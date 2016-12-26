// Copyright 2015, 2016 Parity Technologies (UK) Ltd.
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

export const start = (action, name, address) => ({ type: `reverse ${action} start`, name, address });

export const success = (action) => ({ type: `reverse ${action} success` });

export const fail = (action) => ({ type: `reverse ${action} error` });

export const propose = (name, address) => (dispatch, getState) => {
  const state = getState();
  const account = state.accounts.selected;
  const contract = state.contract;
  if (!contract || !account) {
    return;
  }

  name = name.toLowerCase();

  const proposeReverse = contract.functions.find((f) => f.name === 'proposeReverse');

  dispatch(start('propose', name, address));

  const options = {
    from: account.address
  };
  const values = [
    name,
    address
  ];

  postTx(api, proposeReverse, options, values)
    .then((txHash) => {
      dispatch(success('propose'));
    })
    .catch((err) => {
      console.error(`could not propose reverse ${name} for address ${address}`);
      if (err) {
        console.error(err.stack);
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

  const confirmReverse = contract.functions.find((f) => f.name === 'confirmReverse');

  dispatch(start('confirm', name));

  const options = {
    from: account.address
  };
  const values = [
    name
  ];

  postTx(api, confirmReverse, options, values)
    .then((txHash) => {
      dispatch(success('confirm'));
    })
    .catch((err) => {
      console.error(`could not confirm reverse ${name}`);
      if (err) {
        console.error(err.stack);
      }
      dispatch(fail('confirm'));
    });
};
