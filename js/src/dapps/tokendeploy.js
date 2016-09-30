// Copyright 2015, 2016 Ethcore (UK) Ltd.
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
import { createHashHistory } from 'history';
import { Redirect, Router, Route, useRouterHistory } from 'react-router';

import injectTapEventPlugin from 'react-tap-event-plugin';
injectTapEventPlugin();

import Deploy from './tokendeploy/Deploy';
import Events from './tokendeploy/Events';
import Application from './tokendeploy/Application';
import Overview from './tokendeploy/Overview';
import Send from './tokendeploy/Send';
import Status from './tokendeploy/Status';

const routerHistory = useRouterHistory(createHashHistory)({});

import './style.css';
import './tokendeploy.html';

ReactDOM.render(
  <Router history={ routerHistory }>
    <Redirect from='/' to='/overview' />
    <Route path='/' component={ Application }>
      <Route path='deploy' component={ Deploy } />
      <Route path='events' component={ Events } />
      <Route path='overview' component={ Overview } />
      <Route path='send' component={ Send } />
      <Route path='status' component={ Status } />
    </Route>
  </Router>,
  document.querySelector('#container')
);
