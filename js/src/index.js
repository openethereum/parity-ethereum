import ReactDOM from 'react-dom';
import React from 'react';

import injectTapEventPlugin from 'react-tap-event-plugin';
injectTapEventPlugin();

import { createHashHistory } from 'history';
import { Provider } from 'react-redux';
import { combineReducers, createStore } from 'redux';
import { Redirect, Router, Route, useRouterHistory } from 'react-router';
import MuiThemeProvider from 'material-ui/styles/MuiThemeProvider';

import { muiTheme } from './ui';
import { Accounts, Account, Application, Contract, Contracts, Dapp, Dapps } from './views';
import Signer from './views/Signer/containers/Root';

import { errorReducer } from './ui/Errors';
import { tooltipReducer } from './ui/Tooltips';
import { statusReducer } from './views/Application/Status';
import { signer as signerReducer, toastr as signerToastrReducer, requests as signerRequestsReducer } from './views/Signer/reducers';

import styles from './reset.css';
import './index.html';

const reducers = combineReducers({
  errors: errorReducer,
  status: statusReducer,
  tooltip: tooltipReducer,
  signer: signerReducer,
  toastr: signerToastrReducer,
  requests: signerRequestsReducer
});
const store = createStore(reducers, {});
const routerHistory = useRouterHistory(createHashHistory)({});

ReactDOM.render(
  <Provider store={ store }>
    <MuiThemeProvider muiTheme={ muiTheme }>
      <Router className={ styles.reset } history={ routerHistory }>
        <Redirect from='/' to='/accounts' />
        <Route path='/' component={ Application }>
          <Route path='accounts' component={ Accounts } />
          <Route path='account/:address' component={ Account } />
          <Route path='apps' component={ Dapps } />
          <Route path='app/:name' component={ Dapp } />
          <Route path='contracts' component={ Contracts } />
          <Route path='contract/:address' component={ Contract } />
          <Route path='signer' component={ Signer } />
        </Route>
      </Router>
    </MuiThemeProvider>
  </Provider>,
  document.querySelector('#container')
);
