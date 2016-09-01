
import { handleActions } from 'redux-actions';

const isProd = process.env.NODE_ENV === 'production';

const initialState = {
  logging: !isProd
};

export default handleActions({

  'update logging' (state, action) {
    return {
      logging: action.payload
    };
  }

}, initialState);
