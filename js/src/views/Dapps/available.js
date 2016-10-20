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
// along with Parity. If not, see <http://www.gnu.org/licenses/>.

// TODO remove this hardcoded list of apps once the route works again.

import { sha3 } from '../../api/util/sha3';

const hardcoded = [
  {
    id: 'basiccoin',
    name: 'Token Deployment',
    description: 'Deploy new basic tokens that you are able to send around',
    author: 'Ethcore <admin@ethcore.io>',
    version: '1.0.0'
  },
  {
    id: 'gavcoin',
    name: 'GAVcoin',
    description: 'Manage your GAVcoins, the hottest new property in crypto',
    author: 'Ethcore <admin@ethcore.io>',
    version: '1.0.0'
  },
  {
    id: 'registry',
    name: 'Registry',
    description: 'A global registry of addresses on the network',
    author: 'Ethcore <admin@ethcore.io>',
    version: '1.0.0'
  },
  {
    id: 'tokenreg',
    name: 'Token Registry',
    description: 'A registry of transactable tokens on the network',
    author: 'Ethcore <admin@ethcore.io>',
    version: '1.0.0'
  },
  {
    id: 'signaturereg',
    name: 'Method Registry',
    description: 'A registry of method signatures for lookups on transactions',
    author: 'Ethcore <admin@ethcore.io>',
    version: '1.0.0'
  },
  {
    id: 'githubhint',
    name: 'GitHub Hint',
    description: 'A mapping of GitHub URLs to hashes for use in contracts as references',
    author: 'Ethcore <admin@ethcore.io>',
    version: '1.0.0'
  }
];

export default function () {
  // return fetch('//127.0.0.1:8080/api/apps')
  // .then((res) => res.ok ? res.json() : [])
  return Promise.resolve(hardcoded) // TODO
  .then((apps) => apps.map((app) => {
    return Object.assign({}, app, { hash: sha3(app.id) });
  }));
}
