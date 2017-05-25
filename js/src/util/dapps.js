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

import BigNumber from 'bignumber.js';
import { pick, range, uniq } from 'lodash';

import Contracts from '~/contracts';
import { hashToImageUrl } from '~/redux/util';
import { bytesToHex } from '~/api/util/format';

import builtinJson from '~/views/Dapps/builtin.json';

const builtinApps = builtinJson.filter((app) => app.id);

export function subscribeToChanges (api, dappReg, callback) {
  return dappReg
    .getContract()
    .then((dappRegContract) => {
      const dappRegInstance = dappRegContract.instance;

      const signatures = ['MetaChanged', 'OwnerChanged', 'Registered']
        .map((event) => dappRegInstance[event].signature);

      return api.eth
        .newFilter({
          fromBlock: '0',
          toBlock: 'latest',
          address: dappRegInstance.address,
          topics: [ signatures ]
        })
      .then((filterId) => {
        return api
          .subscribe('eth_blockNumber', () => {
            if (filterId > -1) {
              api.eth
                .getFilterChanges(filterId)
                .then((logs) => {
                  return dappRegContract.parseEventLogs(logs);
                })
                .then((events) => {
                  if (events.length === 0) {
                    return [];
                  }

                  // Return uniq IDs which changed meta-data
                  const ids = uniq(events.map((event) => bytesToHex(event.params.id.value)));

                  callback(ids);
                });
            }
          })
          .then((blockSubId) => {
            return {
              block: blockSubId,
              filter: filterId
            };
          });
      });
    });
}

export function fetchBuiltinApps () {
  const { dappReg } = Contracts.get();

  return Promise
    .all(builtinApps.map((app) => dappReg.getImage(app.id)))
    .then((imageIds) => {
      return builtinApps.map((app, index) => {
        app.type = 'builtin';
        app.image = hashToImageUrl(imageIds[index]);
        return app;
      });
    })
    .catch((error) => {
      console.warn('DappsStore:fetchBuiltinApps', error);
    });
}

export function fetchLocalApps (api) {
  return api.parity.dappsList()
    .then((apps) => {
      return apps
        .map((app) => {
          app.type = 'local';
          app.visible = true;
          return app;
        })
        .filter((app) => app.id && !['ui'].includes(app.id));
    })
    .catch((error) => {
      console.warn('DappsStore:fetchLocal', error);
    });
}

export function fetchRegistryAppIds () {
  const { dappReg } = Contracts.get();

  return dappReg
    .count()
    .then((count) => {
      const promises = range(0, count.toNumber()).map((index) => dappReg.at(index));

      return Promise.all(promises);
    })
    .then((appsInfo) => {
      const appIds = appsInfo
        .map(([appId, owner]) => bytesToHex(appId))
        .filter((appId) => {
          return (new BigNumber(appId)).gt(0) && !builtinApps.find((app) => app.id === appId);
        });

      return uniq(appIds);
    })
    .catch((error) => {
      console.warn('DappsStore:fetchRegistryAppIds', error);
    });
}

export function fetchRegistryApp (api, dappReg, appId) {
  return Promise
    .all([
      dappReg.getImage(appId),
      dappReg.getContent(appId),
      dappReg.getManifest(appId)
    ])
    .then(([ imageId, contentId, manifestId ]) => {
      const app = {
        id: appId,
        image: hashToImageUrl(imageId),
        contentHash: bytesToHex(contentId).substr(2),
        manifestHash: bytesToHex(manifestId).substr(2),
        type: 'network',
        visible: true
      };

      return fetchManifest(api, app.manifestHash)
        .then((manifest) => {
          if (manifest) {
            app.manifestHash = null;

            // Add usefull manifest fields to app
            Object.assign(app, pick(manifest, ['author', 'description', 'name', 'version']));
          }

          return app;
        });
    })
    .then((app) => {
      // Keep dapps that has a Manifest File and an Id
      const dapp = (app.manifestHash || !app.id) ? null : app;

      return dapp;
    })
    .catch((error) => {
      console.warn('DappsStore:fetchRegistryApp', error);
    });
}

export function fetchManifest (api, manifestHash) {
  if (/^(0x)?0+/.test(manifestHash)) {
    return Promise.resolve(null);
  }

  return fetch(
      `/api/content/${manifestHash}/`,
      { redirect: 'follow', mode: 'cors' }
    )
    .then((response) => {
      return response.ok
        ? response.json()
        : null;
    })
    .then((manifest) => {
      return manifest;
    })
    .catch((error) => {
      console.warn('DappsStore:fetchManifest', error);
      return null;
    });
}
