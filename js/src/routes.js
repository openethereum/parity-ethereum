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
import { Application, Dapp, Dapps, Signer, Web } from '~/views';
import builtinDapps from '~/config/dappsBuiltin.json';
import viewsDapps from '~/config/dappsViews.json';

const dapps = [].concat(viewsDapps, builtinDapps);

// const accountsHistory = HistoryStore.get('accounts');
const dappsHistory = HistoryStore.get('dapps');

function redirectTo (path) {
  return (nextState, replace) => {
    replace(path);
  };
}

// const accountsRoutes = [
//   {
//     path: ':address',
//     component: Account,
//     onEnter: ({ params }) => {
//       accountsHistory.add(params.address, 'account');
//     }
//   },
//   {
//     path: '/wallet/:address',
//     component: Wallet,
//     onEnter: ({ params }) => {
//       accountsHistory.add(params.address, 'wallet');
//     }
//   }
// ];

// const contractsRoutes = [
//   { path: ':address', component: Contract }
// ];

const routes = [
  { path: '/', onEnter: redirectTo('/apps') },
  { path: '/auth', onEnter: redirectTo('/apps') }
];

const childRoutes = [
  {
    path: 'app/:id',
    component: Dapp,
    onEnter: ({ params }) => {
      if (!dapps[params.id] || !dapps[params.id].skipHistory) {
        dappsHistory.add(params.id);
      }
    }
  },
  { path: 'apps', component: Dapps },
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
