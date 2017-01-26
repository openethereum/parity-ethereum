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

import { handleActions } from 'redux-actions';

const initialState = {
  finished: [],
  pending: []
};

function removeWithId (pending, id) {
  return pending.filter(tx => tx.id !== id).slice();
}

function setIsSending (pending, id, isSending) {
  return pending.map(p => {
    if (p.id === id) {
      p.isSending = isSending;
    }
    return p;
  }).slice();
}

export default handleActions({
  signerRequestsToConfirm (state, action) {
    const { pending } = action;

    return Object.assign({}, state, {
      pending: pending.map((request) => {
        const { id } = request;
        const oldRequest = state.pending.find((r) => r.id === id);

        request.date = (oldRequest && oldRequest.date)
          ? oldRequest.date
          : new Date();

        return request;
      })
    });
  },

  signerStartConfirmRequest (state, action) {
    return {
      ...state,
      pending: setIsSending(state.pending, action.payload.id, true)
    };
  },

  signerSuccessConfirmRequest (state, action) {
    const { id, txHash } = action.payload;
    const confirmed = Object.assign(
      state.pending.find(p => p.id === id) || { id },
      { result: txHash, status: 'confirmed' }
    );

    return {
      ...state,
      pending: removeWithId(state.pending, id),
      finished: [confirmed].concat(state.finished)
    };
  },

  signerErrorConfirmRequest (state, action) {
    return {
      ...state,
      pending: setIsSending(state.pending, action.payload.id, false)
    };
  },

  signerStartRejectRequest (state, action) {
    return {
      ...state,
      pending: setIsSending(state.pending, action.payload.id, true)
    };
  },

  signerSuccessRejectRequest (state, action) {
    const { id } = action.payload;
    const rejected = Object.assign(
      state.pending.find(p => p.id === id) || { id },
      { status: 'rejected' }
    );

    return {
      ...state,
      pending: removeWithId(state.pending, id),
      finished: [rejected].concat(state.finished)
    };
  },

  signerErrorRejectRequest (state, action) {
    return {
      ...state,
      pending: setIsSending(state.pending, action.payload.id, false)
    };
  }

}, initialState);
