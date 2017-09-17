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
import { AppContainer } from 'react-hot-loader';

import injectTapEventPlugin from 'react-tap-event-plugin';
import { hashHistory } from 'react-router';
import qs from 'querystring';

import SecureApi from './secureApi';
import ContractInstances from '~/contracts';

import { initStore } from './redux';
import ContextProvider from '~/ui/ContextProvider';
import muiTheme from '~/ui/Theme';
import MainApplication from './main';

import { loadSender, patchApi } from '~/util/tx';

import './environment';

import '../assets/fonts/Roboto/font.css';
import '../assets/fonts/RobotoMono/font.css';

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

patchApi(api);
loadSender(api);
ContractInstances.create(api);

const store = initStore(api, hashHistory);

window.secureApi = api;

ReactDOM.render(
  <AppContainer>
    <ContextProvider api={ api } muiTheme={ muiTheme } store={ store }>
      <MainApplication
        routerHistory={ hashHistory }
      />
    </ContextProvider>
  </AppContainer>,
  document.querySelector('#container')
);

if (module.hot) {
  module.hot.accept('./main.js', () => {
    require('./main.js');

    ReactDOM.render(
      <AppContainer>
        <ContextProvider api={ api } muiTheme={ muiTheme } store={ store }>
          <MainApplication
            routerHistory={ hashHistory }
          />
        </ContextProvider>
      </AppContainer>,
      document.querySelector('#container')
    );
  });
}
