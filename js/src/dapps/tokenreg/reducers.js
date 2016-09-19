import { combineReducers } from 'redux';

import status from './Status/reducer';
import tokens from './Tokens/reducer';
import actions from './Actions/reducer';
import accounts from './Accounts/reducer';

const rootReducer = combineReducers({
  status, tokens, actions, accounts
});

export default rootReducer;
