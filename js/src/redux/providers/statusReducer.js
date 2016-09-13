import { handleActions } from 'redux-actions';

const initialState = {
  devLogs: [],
  devLogsLevels: null,
  devLogsEnabled: false
};

export default handleActions({
  statusBlockNumber (state, action) {
    const { blockNumber } = action;

    return Object.assign({}, state, { blockNumber });
  },

  statusCollection (state, action) {
    const { collection } = action;

    return Object.assign({}, state, collection);
  },

  statusLogs (state, action) {
    const { logInfo } = action;

    return Object.assign({}, state, logInfo);
  },

  toggleStatusLogs (state, action) {
    const { devLogsEnabled } = action;

    return Object.assign({}, state, { devLogsEnabled });
  },

  clearStatusLogs (state, action) {
    return Object.assign({}, state, { devLogs: [] });
  }
}, initialState);
