import { applyMiddleware, createStore } from 'redux';

import initMiddleware from './middleware';
import initReducers from './reducers';

import { Balances as BalancesProvider, Personal as PersonalProvider } from './providers';

const storeCreation = window.devToolsExtension
  ? window.devToolsExtension()(createStore)
  : createStore;

export default function (api, signerWs, signerTokenSetter, statusWeb3) {
  const reducers = initReducers();
  const middleware = initMiddleware(signerWs, signerTokenSetter, statusWeb3);
  const store = applyMiddleware(...middleware)(storeCreation)(reducers);

  new PersonalProvider(store, api).start();
  new BalancesProvider(store, api).start();

  return store;
}
