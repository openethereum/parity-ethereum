
import { createAction } from 'redux-actions';

export const error = createAction('error');
export const updateDevLogs = createAction('update devLogs');
export const removeDevLogs = createAction('remove devLogs');
export const updateDevLogging = createAction('update devLogging');
export const updateDevLogsLevels = createAction('update devLogsLevels');
