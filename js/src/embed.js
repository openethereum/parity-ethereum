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

import SecureApi from '~/secureApi';
import ContractInstances from '~/contracts';

import { initStore } from '~/redux';
import ContextProvider from '~/ui/ContextProvider';
import muiTheme from '~/ui/Theme';

import { patchApi } from '~/util/tx';
import { setApi } from '~/redux/providers/apiActions';

import '~/environment';

import '../assets/fonts/Roboto/font.css';
import '../assets/fonts/RobotoMono/font.css';

injectTapEventPlugin();

import ParityBar from '~/views/ParityBar';

// Test transport (std transport should be provided as global object)
class FakeTransport {
  constructor () {
    console.warn('Secure Transport not provided. Falling back to FakeTransport');
  }

  execute (method, ...params) {
    console.log('Calling', method, params);
    return Promise.reject('not connected');
  }

  addMiddleware () {
  }

  on () {
  }
}

class FrameSecureApi extends SecureApi {
  constructor (transport) {
    super(
      transport.uiUrl,
      null,
      () => transport,
      () => 'http:'
    );
  }

  connect () {
    // Do nothing - this API does not need connecting
    this.emit('connecting');
    // Fetch settings
    this._fetchSettings();
    // Fire connected event with some delay.
    setTimeout(() => {
      this.emit('connected');
    });
  }

  needsToken () {
    return false;
  }

  isConnecting () {
    return false;
  }

  isConnected () {
    return true;
  }
}

const transport = window.secureTransport || new FakeTransport();
const uiUrl = transport.uiUrl || 'http://127.0.0.1:8180';

transport.uiUrlWithProtocol = uiUrl;
transport.uiUrl = uiUrl.replace('http://', '').replace('https://', '');
const api = new FrameSecureApi(transport);

patchApi(api);
ContractInstances.create(api);

const store = initStore(api, null, true);

store.dispatch({ type: 'initAll', api });
store.dispatch(setApi(api));

window.secureApi = api;

const app = (
  <ParityBar dapp externalLink={ uiUrl } />
);
const container = document.querySelector('#container');

ReactDOM.render(
  <AppContainer>
    <ContextProvider
      api={ api }
      muiTheme={ muiTheme }
      store={ store }
    >
      { app }
    </ContextProvider>
  </AppContainer>,
  container
);
