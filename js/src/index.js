import 'isomorphic-fetch';
import React from 'react';
import ReactDOM from 'react-dom';
import injectTapEventPlugin from 'react-tap-event-plugin';
import es6Promise from 'es6-promise';

import { createHashHistory } from 'history';
import { Provider } from 'react-redux';
import { applyMiddleware, combineReducers, createStore } from 'redux';
import { Redirect, Router, Route, useRouterHistory } from 'react-router';
import { routerReducer } from 'react-router-redux';
import MuiThemeProvider from 'material-ui/styles/MuiThemeProvider';

import { muiTheme } from './ui';
import { Accounts, Account, Addresses, Address, Application, Contract, Contracts, Dapp, Dapps, Signer } from './views';

import { errorReducer } from './ui/Errors';
import { tooltipReducer } from './ui/Tooltips';
import { nodeStatusReducer } from './views/Application/Status';

// TODO: This is VERY messy, just dumped here to get the Signer going
import signerMiddlewares from './views/Signer/middlewares';
import { signer as signerReducer, toastr as signerToastrReducer, requests as signerRequestsReducer } from './views/Signer/reducers';
import Web3 from 'web3';
import { Web3Provider, web3Extension } from './views/Signer/components';
import { WebSocketsProvider, Ws } from './views/Signer/utils';
import { SignerDataProvider, WsDataProvider } from './views/Signer/providers';

// TODO: same with Status...
import { status as statusReducer, settings as statusSettingsReducer, mining as statusMiningReducer, rpc as statusRpcReducer, toastr as statusToastrReducer, logger as statusLoggerReducer, debug as statusDebugReducer } from './views/Status/reducers';
import Status from './views/Status/containers/Container';

import './environment';

import styles from './reset.css';
import './index.html';

es6Promise.polyfill();
injectTapEventPlugin();

const initToken = window.localStorage.getItem('sysuiToken');
const parityUrl = process.env.NODE_ENV === 'production' ? window.location.host : '127.0.0.1:8180';
const routerHistory = useRouterHistory(createHashHistory)({});
const ws = new Ws(parityUrl);
const web3 = new Web3(new WebSocketsProvider(ws));
web3Extension(web3).map((extension) => web3._extend(extension));

function tokenSetter (token, cb) {
  window.localStorage.setItem('sysuiToken', token);
}

const reducers = combineReducers({
  errors: errorReducer,
  nodeStatus: nodeStatusReducer,
  tooltip: tooltipReducer,
  routing: routerReducer,
  signer: signerReducer,
  signerRequests: signerRequestsReducer,
  signerToastr: signerToastrReducer,
  status: statusReducer,
  statusSettings: statusSettingsReducer,
  statusMining: statusMiningReducer,
  statusRpc: statusRpcReducer,
  statusToastr: statusToastrReducer,
  statusLogger: statusLoggerReducer,
  statusDebug: statusDebugReducer
});
const store = applyMiddleware(...signerMiddlewares(ws, tokenSetter))(
  window.devToolsExtension
    ? window.devToolsExtension()(createStore)
    : createStore
)(reducers);

new WsDataProvider(store, ws); // eslint-disable-line no-new
new SignerDataProvider(store, ws); // eslint-disable-line no-new
ws.init(initToken);

ReactDOM.render(
  <Provider store={ store }>
    <MuiThemeProvider muiTheme={ muiTheme }>
      <Web3Provider web3={ web3 }>
        <Router className={ styles.reset } history={ routerHistory }>
          <Redirect from='/' to='/accounts' />
          <Route path='/' component={ Application }>
            <Route path='accounts' component={ Accounts } />
            <Route path='account/:address' component={ Account } />
            <Route path='addresses' component={ Addresses } />
            <Route path='address/:address' component={ Address } />
            <Route path='apps' component={ Dapps } />
            <Route path='app/:name' component={ Dapp } />
            <Route path='contracts' component={ Contracts } />
            <Route path='contract/:address' component={ Contract } />
            <Route path='signer' component={ Signer } />
            <Route path='status' component={ Status } />
          </Route>
        </Router>
      </Web3Provider>
    </MuiThemeProvider>
  </Provider>,
  document.querySelector('#container')
);
