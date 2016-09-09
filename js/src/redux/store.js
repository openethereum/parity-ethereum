import { applyMiddleware, createStore } from 'redux';

import initMiddleware from './middleware';
import initReducers from './reducers';

const storeCreation = window.devToolsExtension
  ? window.devToolsExtension()(createStore)
  : createStore;

export default function (signerWs, signerTokenSetter, statusWeb3) {
  const reducers = initReducers();
  const middleware = initMiddleware(signerWs, signerTokenSetter, statusWeb3);

  return applyMiddleware(...middleware)(storeCreation)(reducers);
}
