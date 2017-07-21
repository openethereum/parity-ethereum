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
import thunk from 'redux-thunk';
import { routerMiddleware } from 'react-router-redux';

import ErrorsMiddleware from '@parity/ui/Errors/middleware';

import SettingsMiddleware from './providers/settings/middleware';
import SignerMiddleware from './providers/signerMiddleware';

import CertificationsMiddleware from './providers/certifications/middleware';
import ChainMiddleware from './providers/chainMiddleware';
import RegistryMiddleware from './providers/registry/middleware';

export default function (api, browserHistory, forEmbed = false) {
  const middleware = [
    new SettingsMiddleware().toMiddleware(),
    new SignerMiddleware(api).toMiddleware(),
    new ErrorsMiddleware().toMiddleware(),
    new ChainMiddleware().toMiddleware()
  ];

  if (!forEmbed) {
    middleware.push(
      new CertificationsMiddleware(api).toMiddleware(),
      new RegistryMiddleware(api).toMiddleware()
    );
  }

  return middleware.concat(
    browserHistory
      ? routerMiddleware(browserHistory)
      : [],
    thunk
  );
}
