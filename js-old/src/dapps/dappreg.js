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
import injectTapEventPlugin from 'react-tap-event-plugin';
import { AppContainer } from 'react-hot-loader';

injectTapEventPlugin();

import Application from './dappreg/Application';

import '../../assets/fonts/Roboto/font.css';
import '../../assets/fonts/RobotoMono/font.css';
import './style.css';

ReactDOM.render(
  <AppContainer>
    <Application />
  </AppContainer>,
  document.querySelector('#container')
);

if (module.hot) {
  module.hot.accept('./dappreg/Application/index.js', () => {
    require('./dappreg/Application/index.js');

    ReactDOM.render(
      <AppContainer>
        <Application />
      </AppContainer>,
      document.querySelector('#container')
    );
  });
}
