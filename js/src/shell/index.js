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

import 'babel-polyfill';
import 'whatwg-fetch';

import es6Promise from 'es6-promise';
es6Promise.polyfill();

import React from 'react';
import ReactDOM from 'react-dom';

import injectTapEventPlugin from 'react-tap-event-plugin';
import { IndexRoute, Redirect, Route, Router, hashHistory } from 'react-router';
import qs from 'querystring';

import SecureApi from '~/secureApi';
import ContractInstances from '~/contracts';

import { initStore } from '~/redux';
import ContextProvider from '~/ui/ContextProvider';
import muiTheme from '~/ui/Theme';
import { patchApi } from '~/util/tx';

import Application from './Application';
import Dapp from './Dapp';
import Dapps from './Dapps';

import '~/environment';

import '~/../assets/fonts/Roboto/font.css';
import '~/../assets/fonts/RobotoMono/font.css';

injectTapEventPlugin();

if (process.env.NODE_ENV === 'development') {
  // Expose the React Performance Tools on the`window` object
  const Perf = require('react-addons-perf');

  window.Perf = Perf;
}

const AUTH_HASH = '#/auth?';
const parityUrl = process.env.PARITY_URL || window.location.host;
const urlScheme = window.location.href.match(/^https/) ? 'wss://' : 'ws://';

let token = null;

if (window.location.hash && window.location.hash.indexOf(AUTH_HASH) === 0) {
  token = qs.parse(window.location.hash.substr(AUTH_HASH.length)).token;
}

const api = new SecureApi(`${urlScheme}${parityUrl}`, token);

patchApi(api);
ContractInstances.get(api);

const store = initStore(api, hashHistory);

window.secureApi = api;

import HistoryStore from '~/mobx/historyStore';
import builtinDapps from '~/config/dappsBuiltin.json';
import viewsDapps from '~/config/dappsViews.json';

const dapps = [].concat(viewsDapps, builtinDapps);

const dappsHistory = HistoryStore.get('dapps');

function onEnterDapp ({ params }) {
  if (!dapps[params.id] || !dapps[params.id].skipHistory) {
    dappsHistory.add(params.id);
  }
}

ReactDOM.render(
  <ContextProvider api={ api } muiTheme={ muiTheme } store={ store }>
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
