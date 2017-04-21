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
import { hashHistory } from 'react-router';

import injectTapEventPlugin from 'react-tap-event-plugin';
injectTapEventPlugin();

import { api } from './parity';

import { initStore } from '~/redux';
import ContextProvider from '~/ui/ContextProvider';
import muiTheme from '~/ui/Theme';

import Home from './home';

import '~/../assets/fonts/Roboto/font.css';
import '~/../assets/fonts/RobotoMono/font.css';

import './home.css';

const store = initStore(api, hashHistory);

ReactDOM.render(
  <ContextProvider api={ api } muiTheme={ muiTheme } store={ store }>
    <Home />
  </ContextProvider>,
  document.querySelector('#container')
);
