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

import React from 'react';
import ReactDOM from 'react-dom';
import { Route, Router, hashHistory } from 'react-router';

/** Application Initialization **/
import injectTapEventPlugin from 'react-tap-event-plugin';
import ContractInstances from '@parity/shared/contracts';
import ContextProvider from '~/ui/ContextProvider';
import { initStore } from '@parity/shared/redux';
import muiTheme from '~/ui/Theme';
import { api } from './parity';
ContractInstances.get(api);
injectTapEventPlugin();

/** Additional Frameworks **/
import 'bootstrap/dist/css/bootstrap.css';

/** Components **/
// import App from './AppStore/App';
import {
  Dapps,
  Home
} from './Components';

/** Stylesheets **/
import './index.css';

/** Globals **/
const store = initStore(api, hashHistory);

// import ReactDOM from 'react-dom';
// import React from 'react';
// import { Route, Router, hashHistory } from 'react-router';
//
// import injectTapEventPlugin from 'react-tap-event-plugin';
// injectTapEventPlugin();
//
// import ContractInstances from '@parity/shared/contracts';
// import { initStore } from '@parity/shared/redux';
//
// import { api } from './parity';
//
// import ContextProvider from '~/ui/ContextProvider';
// import muiTheme from '~/ui/Theme';
//
// import {
//   Dapps,
//   Home
// } from './Components';
//
// ContractInstances.get(api);
//
// const store = initStore(api, hashHistory);

ReactDOM.render(
  <ContextProvider api={ api } muiTheme={ muiTheme } store={ store }>
    <Router history={ hashHistory }>
      <Route path='/' component={ Home } />
      <Route path='/dapps/:appPath' component={ Dapps } />
    </Router>
  </ContextProvider>,
  document.querySelector('#container')
);
