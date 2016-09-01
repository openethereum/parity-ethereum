import ReactDOM from 'react-dom';
import React from 'react';

import injectTapEventPlugin from 'react-tap-event-plugin';
injectTapEventPlugin();

import { createHashHistory } from 'history';
import { Provider } from 'react-redux';
import { combineReducers, createStore } from 'redux';
import { Redirect, Router, Route, useRouterHistory } from 'react-router';
import MuiThemeProvider from 'material-ui/styles/MuiThemeProvider';

import muiTheme from './ui/Theme';
import Accounts from './views/Accounts';
import Account from './views/Account';
import Contracts from './views/Contracts';
import Contract from './views/Contract';
import Dapps from './views/Dapps';
import Dapp from './views/Dapp';
import Signer from './views/Signer';
import Application from './views/Application';

import { errorReducer } from './ui/Errors';
import { tooltipReducer } from './ui/Tooltips';
import { statusReducer } from './views/Application/Status';

import styles from './reset.css';
import './index.html';

const store = createStore(combineReducers({
  errors: errorReducer,
  status: statusReducer,
  tooltip: tooltipReducer
}), {});

const routerHistory = useRouterHistory(createHashHistory)({});

ReactDOM.render(
  <Provider store={ store }>
    <MuiThemeProvider muiTheme={ muiTheme }>
      <Router history={ routerHistory } className={ styles.reset }>
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
