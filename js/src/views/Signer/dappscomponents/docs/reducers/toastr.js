
import { handleActions } from 'redux-actions';

const initialState = {
  toasts: [],
  toastNo: 1
};

export default handleActions({

  'add toast' (state, action) {
    return {
      ...state,
      toastNo: state.toastNo + 1,
      toasts: [action.payload].concat(state.toasts)
    };
  },

  'remove toast' (state, action) {
    return {
      ...state,
      toasts: state.toasts.filter(t => t.toastNo !== action.payload)
    };
  }

}, initialState);
