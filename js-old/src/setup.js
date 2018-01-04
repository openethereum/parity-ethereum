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

import injectTapEventPlugin from 'react-tap-event-plugin';
import { hashHistory } from 'react-router';

import Api from '@parity/api';

import ContractInstances from '~/contracts';

import { initStore } from './redux';
import muiTheme from '~/ui/Theme';

import { loadSender, patchApi } from '~/util/tx';
import { setApi } from '~/redux/providers/apiActions';

import './environment';

import '~/assets/fonts/Roboto/font.css';
import '~/assets/fonts/RobotoMono/font.css';

injectTapEventPlugin();

const api = new Api(window.ethereum);

patchApi(api);
loadSender(api);
ContractInstances.create(api);

const store = initStore(api, hashHistory);

store.dispatch({ type: 'initAll', api });
store.dispatch(setApi(api));

export default {
  api, muiTheme, store, hashHistory
};
