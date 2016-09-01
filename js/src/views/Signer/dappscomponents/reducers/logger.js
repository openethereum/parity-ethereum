import { handleActions } from 'redux-actions';

const initialState = {
  logging: process.env.LOGGING
};

export default handleActions({

  'update logging' (state, action) {
    return {
      logging: action.payload
    };
  }

}, initialState);
