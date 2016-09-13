import { combineReducers } from 'redux';

import status from './Status/reducers';

const rootReducer = combineReducers({
  status
});

export default rootReducer;
