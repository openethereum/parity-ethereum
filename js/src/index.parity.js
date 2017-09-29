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

import React from 'react';
import ReactDOM from 'react-dom';

import injectTapEventPlugin from 'react-tap-event-plugin';
import { IndexRoute, Redirect, Route, Router, hashHistory } from 'react-router';
import qs from 'querystring';

import ContractInstances from '@parity/shared/contracts';
import { initStore } from '@parity/shared/redux';
import ContextProvider from '@parity/ui/ContextProvider';

import '@parity/shared/environment';

import Application from './Application';
import Dapp from './Dapp';
import Dapps from './Dapps';
import { setupProviderFilters } from './DappRequests';
import { injectExternalScript } from './ShellExtend';
import SecureApi from './secureApi';

injectTapEventPlugin();

window.React = window.React || React;

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

const api = new SecureApi(window.location.host, token);

ContractInstances.get(api);

setupProviderFilters(api.provider);

const store = initStore(api, hashHistory);

console.log('UI version', process.env.UI_VERSION);

ReactDOM.render(
  <ContextProvider api={ api } store={ store }>
    <Router history={ hashHistory }>
      <Route path='/' component={ Application }>
        <Redirect from='/auth' to='/' />
        <Route path='/:id' component={ Dapp } />
        <Route path='/:id/:details' component={ Dapp } />
        <IndexRoute component={ Dapps } />
      </Route>
    </Router>
  </ContextProvider>,
  document.querySelector('#container')
);

// testing, priceTicker gist
injectExternalScript('https://cdn.rawgit.com/jacogr/396fc583e81b9404e21195a48dc862ca/raw/33e5058a4c0028cf9acf4b0662d75298e41ca6fa/priceTicker.js');

// testing, signer plugins
import '@parity/plugin-signer-account';
import '@parity/plugin-signer-default';
import '@parity/plugin-signer-hardware';
import '@parity/plugin-signer-qr';
