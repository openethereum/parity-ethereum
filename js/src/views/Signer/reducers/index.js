import { combineReducers } from 'redux';
import { routerReducer as routing } from 'react-router-redux';
import app from './app';
import toastr from './toastr';
import requests from './requests';

export default combineReducers({
  routing,
  app,
  toastr,
  requests
});
