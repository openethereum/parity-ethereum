import ReactDOM from 'react-dom';
import React from 'react';

import injectTapEventPlugin from 'react-tap-event-plugin';
injectTapEventPlugin();

import { Redirect, Router, Route, useRouterHistory } from 'react-router';
import { createHashHistory } from 'history';

import Accounts from './views/Accounts';
import Account from './views/Account';
import Contracts from './views/Contracts';
import Contract from './views/Contract';
import Dapps from './views/Dapps';
import Dapp from './views/Dapp';
import Signer from './views/Signer';
import Application from './views/Application';

import styles from './reset.css';

const routerHistory = useRouterHistory(createHashHistory)({});

ReactDOM.render(
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
  </Router>,
  document.querySelector('#container')
);
