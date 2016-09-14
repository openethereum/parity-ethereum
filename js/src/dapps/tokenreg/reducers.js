import { combineReducers } from 'redux';

import status from './Status/reducers';
import tokens from './Tokens/reducers';

const rootReducer = combineReducers({
  status, tokens
});

export default rootReducer;
