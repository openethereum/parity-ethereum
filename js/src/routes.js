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

import HistoryStore from '~/mobx/historyStore';
import {
  Accounts, Account, Addresses, Address, Application,
  Contract, Contracts, Dapp, Dapps, Home,
  Settings, SettingsBackground, SettingsParity, SettingsProxy,
  SettingsViews, Signer, Status,
  Vaults, Wallet, Web, WriteContract
} from '~/views';
import builtinDapps from '~/views/Dapps/builtin.json';

const accountsHistory = HistoryStore.get('accounts');
const dappsHistory = HistoryStore.get('dapps');

function handleDeprecatedRoute (nextState, replace) {
  const { address } = nextState.params;
  const redirectMap = {
    account: 'accounts',
    address: 'addresses',
    contract: 'contracts'
  };

  const oldRoute = nextState.routes[0].path;
  const newRoute = Object.keys(redirectMap).reduce((newRoute, key) => {
    return newRoute.replace(new RegExp(`^/${key}`), '/' + redirectMap[key]);
  }, oldRoute);

  console.warn(`Route "${oldRoute}" is deprecated. Please use "${newRoute}"`);
  replace(newRoute.replace(':address', address));
}

function redirectTo (path) {
  return (nextState, replace) => {
    replace(path);
  };
}

const accountsRoutes = [
  {
    path: ':address',
    component: Account,
    onEnter: ({ params }) => {
      accountsHistory.add(params.address, 'account');
    }
  },
  { path: '/vaults', component: Vaults },
  {
    path: '/wallet/:address',
    component: Wallet,
    onEnter: ({ params }) => {
      accountsHistory.add(params.address, 'wallet');
    }
  }
];

const addressesRoutes = [
  { path: ':address', component: Address }
];

const contractsRoutes = [
  { path: 'develop', component: WriteContract },
  { path: ':address', component: Contract }
];

const settingsRoutes = [
  { path: 'background', component: SettingsBackground },
  { path: 'proxy', component: SettingsProxy },
  { path: 'views', component: SettingsViews },
  { path: 'parity', component: SettingsParity }
];

const statusRoutes = [
  { path: ':subpage', component: Status }
];

const routes = [
  // Backward Compatible routes
  { path: '/account/:address', onEnter: handleDeprecatedRoute },
  { path: '/address/:address', onEnter: handleDeprecatedRoute },
  { path: '/contract/:address', onEnter: handleDeprecatedRoute },

  { path: '/', onEnter: redirectTo('/home') },
  { path: '/auth', onEnter: redirectTo('/home') },
  { path: '/settings', onEnter: redirectTo('/settings/views') }
];

const childRoutes = [
  {
    path: 'accounts',
    indexRoute: { component: Accounts },
    childRoutes: accountsRoutes
  },
  {
    path: 'addresses',
    indexRoute: { component: Addresses },
    childRoutes: addressesRoutes
  },
  {
    path: 'contracts',
    indexRoute: { component: Contracts },
    childRoutes: contractsRoutes
  },
  {
    path: 'status',
    indexRoute: { component: Status },
    childRoutes: statusRoutes
  },
  {
    path: 'settings',
    component: Settings,
    childRoutes: settingsRoutes
  },
  {
    path: 'app/:id',
    component: Dapp,
    onEnter: ({ params }) => {
      if (!builtinDapps[params.id] || !builtinDapps[params.id].skipHistory) {
        dappsHistory.add(params.id);
      }
    }
  },
  { path: 'apps', component: Dapps },
  { path: 'home', component: Home },
  { path: 'web', component: Web },
  { path: 'web/:url', component: Web },
  { path: 'signer', component: Signer }
];

// TODO : use ES6 imports when supported
if (process.env.NODE_ENV !== 'production') {
  const Playground = require('./playground').default;

  childRoutes.push({
    path: 'playground',
    component: Playground
  });
}

routes.push({
  path: '/',
  component: Application,
  childRoutes
});

export default routes;
