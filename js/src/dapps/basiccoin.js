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
import { Redirect, Router, Route, hashHistory } from 'react-router';

import injectTapEventPlugin from 'react-tap-event-plugin';
injectTapEventPlugin();

import Deploy from './tokendeploy/Deploy';
import Application from './tokendeploy/Application';
import Overview from './tokendeploy/Overview';
import Transfer from './tokendeploy/Transfer';

import '../../assets/fonts/Roboto/font.css';
import '../../assets/fonts/RobotoMono/font.css';
import './style.css';

ReactDOM.render(
  <Router history={ hashHistory }>
    <Redirect from='/' to='/overview' />
    <Route path='/' component={ Application }>
      <Route path='deploy' component={ Deploy } />
      <Route path='overview' component={ Overview } />
      <Route path='transfer' component={ Transfer } />
    </Route>
  </Router>,
  document.querySelector('#container')
);
