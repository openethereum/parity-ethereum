
import { handleActions } from 'redux-actions';

const initialState = {
  toasts: [],
  id: 1
};

export default handleActions({

  'add toast' (state, action) {
    return {
      ...state,
      id: state.id + 1,
      toasts: [action.payload].concat(state.toasts)
    };
  },

  'remove toast' (state, action) {
    return {
      ...state,
      toasts: state.toasts.filter(t => t.id !== action.payload)
    };
  }

}, initialState);
