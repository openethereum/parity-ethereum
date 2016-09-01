import { routerReducer as routing } from 'react-router-redux';
import { combineReducers } from 'redux';
import toastr from './toastr';
import logger from './logger';

export default combineReducers({
  routing,
  toastr,
  logger
});
