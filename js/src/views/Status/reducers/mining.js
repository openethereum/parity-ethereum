
import { handleActions } from 'redux-actions';

const initialState = {
  author: 'loading...',
  extraData: 'loading...',
  defaultExtraData: '0x01',
  minGasPrice: 'loading...',
  gasFloorTarget: 'loading...'
};

export const actionHandlers = {

  'update author' (state, action) {
    return {
      ...state,
      author: `${action.payload}`
    };
  },

  'update minGasPrice' (state, action) {
    return {
      ...state,
      minGasPrice: `${action.payload}`
    };
  },

  'update gasFloorTarget' (state, action) {
    return {
      ...state,
      gasFloorTarget: `${action.payload}`
    };
  },

  'update extraData' (state, action) {
    return {
      ...state,
      extraData: `${action.payload}`
    };
  },

  'update defaultExtraData' (state, action) {
    return {
      ...state,
      defaultExtraData: `${action.payload}`
    };
  }

};

export default handleActions(actionHandlers, initialState);
