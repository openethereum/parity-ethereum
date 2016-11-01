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

import BigNumber from 'bignumber.js';

import { parityNode } from '../../environment';

const builtinApps = [
  {
    id: '0xf9f2d620c2e08f83e45555247146c62185e4ab7cf82a4b9002a265a0d020348f',
    url: 'basiccoin',
    name: 'Token Deployment',
    description: 'Deploy new basic tokens that you are able to send around',
    author: 'Parity Team <admin@ethcore.io>',
    version: '1.0.0'
  },
  {
    id: '0xd1adaede68d344519025e2ff574650cd99d3830fe6d274c7a7843cdc00e17938',
    url: 'registry',
    name: 'Registry',
    description: 'A global registry of addresses on the network',
    author: 'Parity Team <admin@ethcore.io>',
    version: '1.0.0'
  },
  {
    id: '0x0a8048117e51e964628d0f2d26342b3cd915248b59bcce2721e1d05f5cfa2208',
    url: 'tokenreg',
    name: 'Token Registry',
    description: 'A registry of transactable tokens on the network',
    author: 'Parity Team <admin@ethcore.io>',
    version: '1.0.0'
  },
  {
    id: '0xf49089046f53f5d2e5f3513c1c32f5ff57d986e46309a42d2b249070e4e72c46',
    url: 'signaturereg',
    name: 'Method Registry',
    description: 'A registry of method signatures for lookups on transactions',
    author: 'Parity Team <admin@ethcore.io>',
    version: '1.0.0'
  },
  {
    id: '0x058740ee9a5a3fb9f1cfa10752baec87e09cc45cd7027fd54708271aca300c75',
    url: 'githubhint',
    name: 'GitHub Hint',
    description: 'A mapping of GitHub URLs to hashes for use in contracts as references',
    author: 'Parity Team <admin@ethcore.io>',
    version: '1.0.0'
  }
];

// TODO: This is just since we are moving gavcoin to its own repo, for a proper network solution
// we need to pull the network apps from the dapp registry. (Builtins & local apps unaffected)
// TODO: Manifest needs to be pulled from the content as well, however since the content may or may
// not be available locally (and refreshes work for index, and will give a 503), we are putting it
// in here. This part needs to be cleaned up.
const networkApps = [
  {
    id: '0xd798a48831b4ccdbc71de206a1d6a4aa73546c7b6f59c22a47452af414dc64d6',
    name: 'GAVcoin',
    description: 'Manage your GAVcoins, the hottest new property in crypto',
    author: 'Gavin Wood',
    version: '1.0.0'
  }
];

export function fetchAvailable (api) {
  // TODO: Since we don't have an extensive GithubHint app, get the value somehow
  // RESULT: 0x22cd66e1b05882c0fa17a16d252d3b3ee2238ccbac8153f69a35c83f02ca76ee
  // api.ethcore
  //   .hashContent('https://codeload.github.com/gavofyork/gavcoin/zip/5a9f11ff2ad0d05c565a938ceffdfa0d23af9981')
  //   .then((sha3) => {
  //     console.log('archive', sha3);
  //   });

  return fetch(`${parityNode}/api/apps`)
    .then((response) => {
      return response.ok
        ? response.json()
        : [];
    })
    .catch((error) => {
      console.warn('fetchAvailable', error);
      return [];
    })
    .then((_localApps) => {
      const localApps = _localApps
        .filter((app) => !['ui'].includes(app.id))
        .map((app) => {
          app.local = true;
          return app;
        });

      return api.ethcore
        .registryAddress()
        .then((registryAddress) => {
          if (new BigNumber(registryAddress).eq(0)) {
            return [];
          }

          const _builtinApps = builtinApps
            .map((app) => {
              app.builtin = true;
              return app;
            });

          return networkApps
            .map((app) => {
              app.network = true;
              return app;
            })
            .concat(_builtinApps);
        })
        .then((registryApps) => {
          return registryApps
            .concat(localApps)
            .sort((a, b) => (a.name || '').localeCompare(b.name || ''));
        });
    })
    .catch((error) => {
      console.warn('fetchAvailable', error);
    });
}

export function fetchManifest (app, contentHash) {
  return fetch(`${parityNode}/${contentHash}/manifest.json`)
    .then((response) => {
      return response.ok
        ? response.json()
        : {};
    })
    .then((manifest) => {
      Object.keys.forEach((key) => {
        app[key] = manifest[key];
      });

      return app;
    })
    .catch((error) => {
      console.warn('fetchManifest', error);
    });
}
