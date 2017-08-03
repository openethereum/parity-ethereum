// Copyright 2015-2017 Parity Technologies (UK) Ltd.
// This file is part of Parity.

// Parity is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Parity is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Parity.  If not, see <http://www.gnu.org/licenses/>.

import 'whatwg-fetch';

import es6Promise from 'es6-promise';
es6Promise.polyfill();

import React from 'react';
import ReactDOM from 'react-dom';

import injectTapEventPlugin from 'react-tap-event-plugin';
import { IndexRoute, Redirect, Route, Router, hashHistory } from 'react-router';
import qs from 'querystring';

import Api from '@parity/api';
import builtinDapps from '@parity/shared/config/dappsBuiltin.json';
import viewsDapps from '@parity/shared/config/dappsViews.json';
import ContractInstances from '@parity/shared/contracts';
import HistoryStore from '@parity/shared/mobx/historyStore';
import { initStore } from '@parity/shared/redux';
import ContextProvider from '@parity/ui/ContextProvider';

import '@parity/shared/environment';

import Application from './Application';
import Dapp from './Dapp';
import { setupProviderFilters } from './DappRequests';
import Dapps from './Dapps';
import SecureApi from './secureApi';

injectTapEventPlugin();

if (process.env.NODE_ENV === 'development') {
  // Expose the React Performance Tools on the`window` object
  const Perf = require('react-addons-perf');

  window.Perf = Perf;
}

const AUTH_HASH = '#/auth?';

let token = null;

if (window.location.hash && window.location.hash.indexOf(AUTH_HASH) === 0) {
  token = qs.parse(window.location.hash.substr(AUTH_HASH.length)).token;
}

const uiUrl = window.location.host;
const api = new SecureApi(uiUrl, token);

ContractInstances.get(api);

setupProviderFilters(api.provider);

const store = initStore(api, hashHistory);

const dapps = [].concat(viewsDapps, builtinDapps).map((app) => {
  if (app.id && app.id.substr(0, 2) !== '0x') {
    app.id = Api.util.sha3(app.id);
  }

  return app;
});

const dappsHistory = HistoryStore.get('dapps');

function onEnterDapp ({ params: { id } }) {
  if (!dapps[id] || !dapps[id].skipHistory) {
    dappsHistory.add(id);
  }
}

console.log('UI version', process.env.UI_VERSION);
console.log('Loaded dapps', dapps);

ReactDOM.render(
  <ContextProvider api={ api } store={ store }>
    <Router history={ hashHistory }>
      <Route path='/' component={ Application }>
        <Redirect from='/auth' to='/' />
        <Route path='/:id' component={ Dapp } onEnter={ onEnterDapp } />
        <Route path='/:id/:details' component={ Dapp } onEnter={ onEnterDapp } />
        <IndexRoute component={ Dapps } />
      </Route>
    </Router>
  </ContextProvider>,
  document.querySelector('#container')
);
