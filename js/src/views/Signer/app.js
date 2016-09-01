import 'babel-polyfill';

import React from 'react';
import { Provider } from 'react-redux';
import ReactDOM from 'react-dom';

// Needed for onTouchTap
// http://stackoverflow.com/a/34015469/988941
import injectTapEventPlugin from 'react-tap-event-plugin';

import Web3 from 'web3';
import { Web3Provider, MuiThemeProvider, web3Extension } from 'dapps-react-components';

import 'reset-css/reset.css';
import './index.css';
import './utils/logger';

import Ws from './utils/Ws';
import WebSocketsProvider from './utils/Web3WebSockets';

import WsDataProvider from './providers/wsProvider';
import AppDataProvider from './providers/appProvider';

import { updateUrl } from './actions/app';
import middlewares from './middlewares';
import createStore from './store/configureStore';
import Routes from './routes';

export default function app (token, setToken, parityUrl) {
  const ws = new Ws(parityUrl);
  const web3 = new Web3(new WebSocketsProvider(ws));

  web3Extension(web3).map(extension => web3._extend(extension));

  // TODO [todr] Extend and use Web3 instead of ws directly!
  const store = createStore(middlewares(ws, setToken));
  store.dispatch(updateUrl(parityUrl));

  injectTapEventPlugin();

  ReactDOM.render(
    <Provider store={ store }>
      <Web3Provider web3={ web3 }>
        <MuiThemeProvider>
          <Routes store={ store } />
        </MuiThemeProvider>
      </Web3Provider>
    </Provider>,
    document.querySelector('#root')
  );

  new WsDataProvider(store, ws); // eslint-disable-line no-new
  new AppDataProvider(store, ws); // eslint-disable-line no-new

  ws.init(token);
}

// expose globally for Signer Dapp
global.paritySigner = app;
