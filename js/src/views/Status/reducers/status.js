
import { handleActions } from 'redux-actions';

const initialState = {
  error: false,
  noOfErrors: 0,
  name: 'My node',
  bestBlock: 'loading...',
  hashrate: 'loading...',
  connectedPeers: 0,
  activePeers: 0,
  peers: 0,
  accounts: [],
  version: '-'
};

export default handleActions({
  error (state, action) {
    return {
      ...state,
      disconnected: (action.payload.message === 'Invalid JSON RPC response: ""'),
      noOfErrors: state.noOfErrors + 1
    };
  },

  'update blockNumber' (state, action) {
    return {
      ...resetError(state),
      bestBlock: `${action.payload}`
    };
  },

  'update hashrate' (state, action) {
    return {
      ...resetError(state),
      hashrate: `${action.payload}`
    };
  },

  'update netPeers' (state, action) {
    return {
      ...state,
      connectedPeers: action.payload.connected,
      activePeers: action.payload.active
    };
  },

  'update version' (state, action) {
    return {
      ...resetError(state),
      version: action.payload
    };
  },

  'update accounts' (state, action) {
    return {
      ...resetError(state),
      accounts: action.payload
    };
  },

  'update nodeName' (state, action) {
    return {
      ...resetError(state),
      name: action.payload || ' '
    };
  }

}, initialState);

function resetError (state) {
  return {
    ...state,
    disconnected: false,
    noOfErrors: 0
  };
}
