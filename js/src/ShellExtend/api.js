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

import InterceptorStore from '../DappRequests/store';
import SignerPluginStore from '../Signer/pluginStore';
import StatusPluginStore from '../Status/pluginStore';

function injectInterceptorPlugin (middleware) {
  return InterceptorStore.get().addMiddleware(middleware);
}

function injectSignerPlugin (component, isHandler) {
  let isDefault;

  try {
    isDefault = isHandler(null, null, null) || false;
  } catch (error) {
    isDefault = false;
  }

  return SignerPluginStore.get().addComponent(component, isHandler, isDefault);
}

function injectStatusPlugin (component) {
  return StatusPluginStore.get().addComponent(component);
}

export function extendShell (options) {
  switch (options.type) {
    case 'interceptor':
      return injectInterceptorPlugin(options.middleware);

    case 'signer':
      return injectSignerPlugin(options.component, options.isHandler);

    case 'status':
      return injectStatusPlugin(options.component);

    default:
      throw new Error(`Unable to extend the shell with type '${options.type}'`);
  }
}
