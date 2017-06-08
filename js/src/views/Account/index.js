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

import ReactDOM from 'react-dom';
import React from 'react';
import { Route, Router, hashHistory } from 'react-router';

import injectTapEventPlugin from 'react-tap-event-plugin';
injectTapEventPlugin();

import ContractInstances from '@parity/shared/contracts';
import { initStore } from '@parity/shared/redux';
import ContextProvider from '@parity/ui/ContextProvider';

import api from './api';

import Account from './account';

ContractInstances.get(api);

const store = initStore(api, hashHistory);

ReactDOM.render(
  <ContextProvider api={ api } store={ store }>
    <Router history={ hashHistory }>
      <Route path='/:address' component={ Account } />
    </Router>
  </ContextProvider>,
  document.querySelector('#container')
);
