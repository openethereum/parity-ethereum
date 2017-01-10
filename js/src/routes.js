// Copyright 2015, 2016 Parity Technologies (UK) Ltd.
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

import {
  Accounts, Account, Addresses, Address, Application,
  Contract, Contracts, Dapp, Dapps,
  Settings, SettingsBackground, SettingsParity, SettingsProxy,
  SettingsViews, Signer, Status,
  Wallet, Web, WriteContract
} from '~/views';

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
  { path: ':address', component: Account },
  { path: '/wallet/:address', component: Wallet }
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

  { path: '/', onEnter: redirectTo('/accounts') },
  { path: '/auth', onEnter: redirectTo('/accounts') },
  { path: '/settings', onEnter: redirectTo('/settings/views') },

  {
    path: '/',
    component: Application,
    childRoutes: [
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

      { path: 'apps', component: Dapps },
      { path: 'app/:id', component: Dapp },
      { path: 'web', component: Web },
      { path: 'web/:url', component: Web },
      { path: 'signer', component: Signer }
    ]
  }
];

export default routes;
