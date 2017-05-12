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
// Will Turner: You knew my father.
// Pintel: Old Bootstrap Bill? Aye, we knew 'im. Never sat well with ol' Bootstrap, what we did to Sparrow and all. That's why he sent a piece of the treasure off to you, as it were. He said that we deserved to be cursed... and remain cursed. A' course, that didnt sit too well with the captain.
// Ragetti: No, that didn't sit to well with the cap'n at all... Tell 'im what Barbossa did.
// Pintel: [angry] I'M TELLIN' THE STORY. So, what Barbossa did is, he tied a cannon to Bootstrap's bootstraps.
// Ragetti: [snickering quietly] Bootstrap's bootstraps...
// Pintel: And the last we saw of ol' Bill Turner, he was sinkin' into the crushing black oblivion of Davy Jones' Locker. Course, it was only after that we learned we needed his blood to lift the curse.
// Ragetti: Now that's what you'd call ironic.
import 'bootstrap/dist/css/bootstrap.css';

/** Components **/
// import App from './AppStore/App';
import {
  Dapps,
  Home
} from './Components';

/** Globals **/
const store = initStore(api, hashHistory);

ReactDOM.render(
  <ContextProvider api={ api } muiTheme={ muiTheme } store={ store }>
    <Router history={ hashHistory }>
      <Route path='/' component={ Home } />
      <Route path='/dapps/:appPath' component={ Dapps } />
    </Router>
  </ContextProvider>,
  document.querySelector('#container')
);
