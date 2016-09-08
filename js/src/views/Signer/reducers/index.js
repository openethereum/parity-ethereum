import { combineReducers } from 'redux';
import { routerReducer as routing } from 'react-router-redux';
import signer from './signer';
import requests from './requests';

export default combineReducers({
  routing,
  signer,
  requests
});

export {
  signer,
  requests
};
