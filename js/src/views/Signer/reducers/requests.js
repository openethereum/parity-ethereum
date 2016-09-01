
import { handleActions } from 'redux-actions';

const initialState = {
  compatibilityMode: false,
  pending: [],
  finished: []
};

export default handleActions({

  // TODO [legacy;todr] Remove
  'update compatibilityMode' (state, action) {
    return {
      ...state,
      compatibilityMode: action.payload
    };
  },

  'update pendingRequests' (state, action) {
    return {
      ...state,
      pending: action.payload
    };
  },

  'start confirmRequest' (state, action) {
    return {
      ...state,
      pending: setIsSending(state.pending, action.payload.id, true)
    };
  },

  'success confirmRequest' (state, action) {
    const { id, txHash } = action.payload;
    const confirmed = Object.assign(
      state.pending.find(p => p.id === id),
      { result: txHash, status: 'confirmed' }
    );

    return {
      ...state,
      pending: removeWithId(state.pending, id),
      finished: [confirmed].concat(state.finished)
    };
  },

  'error confirmRequest' (state, action) {
    return {
      ...state,
      pending: setIsSending(state.pending, action.payload.id, false)
    };
  },

  'start rejectRequest' (state, action) {
    return {
      ...state,
      pending: setIsSending(state.pending, action.payload.id, true)
    };
  },

  'success rejectRequest' (state, action) {
    const { id } = action.payload;
    const rejected = Object.assign(
      state.pending.find(p => p.id === id),
      { status: 'rejected' }
    );
    return {
      ...state,
      pending: removeWithId(state.pending, id),
      finished: [rejected].concat(state.finished)
    };
  },

  'error rejectRequest' (state, action) {
    return {
      ...state,
      pending: setIsSending(state.pending, action.payload.id, false)
    };
  }

}, initialState);

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
