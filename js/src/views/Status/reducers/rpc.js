
import { handleActions } from 'redux-actions';
import rpcMetods from '../data/rpc.json';

const initialState = {
  prevCalls: [],
  callNo: 1,
  selectedMethod: rpcMetods.methods[0]
};

export const actionHandlers = {

  'add rpcResponse' (state, action) {
    const calls = [action.payload].concat(state.prevCalls);
    const maxCalls = 64;
    return {
      ...state,
      callNo: state.callNo + 1,
      prevCalls: calls.slice(0, maxCalls)
    };
  },

  'sync rpcStateFromLocalStorage' (state, action) {
    return {
      ...state,
      prevCalls: action.payload.prevCalls,
      callNo: action.payload.callNo,
      selectedMethod: action.payload.selectedMethod
    };
  },

  'reset rpcPrevCalls' (state, action) {
    return {
      ...state,
      callNo: 1,
      prevCalls: []
    };
  },

  'select rpcMethod' (state, action) {
    return {
      ...state,
      selectedMethod: action.payload
    };
  }

};

export default handleActions(actionHandlers, initialState);
