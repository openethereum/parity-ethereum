import { combineReducers } from 'redux';
import { routerReducer as routing } from 'react-router-redux';
import signer from './signer';
import toastr from './toastr';
import requests from './requests';

export default combineReducers({
  routing,
  signer,
  toastr,
  requests
});

export {
  signer,
  toastr,
  requests
};
