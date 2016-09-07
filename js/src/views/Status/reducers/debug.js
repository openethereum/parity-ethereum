
import { handleActions } from 'redux-actions';
import { union } from 'lodash';

const initialState = {
  levels: '',
  logging: true,
  logs: []
};

const maxLogs = 1024;

export const actionHandlers = {

  'update devLogsLevels' (state, action) {
    return {
      ...state,
      levels: `${action.payload}`
    };
  },

  'remove devLogs' (state, action) {
    return {
      ...state,
      logs: []
    };
  },

  'update devLogging' (state, action) {
    return {
      ...state,
      logging: action.payload
    };
  },

  'update devLogs' (state, action) {
    if (!state.logging) {
      return { ...state };
    }

    let newLogs = union(state.logs, action.payload.reverse());

    return {
      ...state,
      logs: newLogs.slice(newLogs.length - maxLogs)
    };
  }

};

export default handleActions(actionHandlers, initialState);
