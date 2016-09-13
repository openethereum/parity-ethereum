import { handleActions } from 'redux-actions';

const initialState = {
};

export default handleActions({
  getBalances (state, action) {
    const { balances } = action;

    return Object.assign({}, state, { balances });
  },

  getTokens (state, action) {
    const { tokens } = action;

    return Object.assign({}, state, { tokens });
  }
}, initialState);
