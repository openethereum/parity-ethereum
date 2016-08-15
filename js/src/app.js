import ReactDOM from 'react-dom';
import React from 'react';

import 'isomorphic-fetch';

import es6Promise from 'es6-promise';
es6Promise.polyfill();

import injectTapEventPlugin from 'react-tap-event-plugin';
injectTapEventPlugin();

import { Redirect, Router, Route, useRouterHistory } from 'react-router';
import { createHashHistory } from 'history';

import Accounts from './views/Accounts';
import Account from './views/Account';
import Application from './views/Application';
import Apps from './views/Apps';
import Tokens from './views/Tokens';

import styles from './reset.css';

const routerHistory = useRouterHistory(createHashHistory)({});

ReactDOM.render(
  <Router history={ routerHistory } className={ styles.reset }>
    <Redirect from='/' to='/accounts' />
    <Route path='/' component={ Application }>
      <Route path='accounts' component={ Accounts } />
      <Route path='account/:address' component={ Account } />
      <Route path='apps' component={ Apps } />
      <Route path='tokens' component={ Tokens } />
    </Route>
  </Router>,
  document.querySelector('#container')
);
