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

import { uniq, range, debounce } from 'lodash';

import { addCertification, removeCertification } from './actions';

import { getLogger, LOG_KEYS } from '~/config';
import Contract from '~/api/contract';
import Contracts from '~/contracts';
import CertifierABI from '~/contracts/abi/certifier.json';

const log = getLogger(LOG_KEYS.CertificationsMiddleware);

// TODO: move this to a more general place
const updatableFilter = (api, onFilter) => {
  let filter = null;

  const update = (address, topics) => {
    if (filter) {
      filter = filter.then((filterId) => {
        api.eth.uninstallFilter(filterId);
      });
    }

    filter = (filter || Promise.resolve())
      .then(() => api.eth.newFilter({
        fromBlock: 0,
        toBlock: 'latest',
        address,
        topics
      }))
      .then((filterId) => {
        onFilter(filterId);
        return filterId;
      })
      .catch((err) => {
        console.error('Failed to create certifications filter:', err);
      });

    return filter;
  };

  return update;
};

export default class CertificationsMiddleware {
  toMiddleware () {
    const api = Contracts.get()._api;
    const badgeReg = Contracts.get().badgeReg;

    const contract = new Contract(api, CertifierABI);
    const Confirmed = contract.events.find((e) => e.name === 'Confirmed');
    const Revoked = contract.events.find((e) => e.name === 'Revoked');

    return (store) => {
      let certifiers = [];
      let addresses = [];
      let filterChanged = false;
      let filter = null;
      let badgeRegFilter = null;
      let fetchCertifiersPromise = null;

      const updateFilter = updatableFilter(api, (filterId) => {
        filterChanged = true;
        filter = filterId;
      });

      const badgeRegUpdateFilter = updatableFilter(api, (filterId) => {
        filterChanged = true;
        badgeRegFilter = filterId;
      });

      badgeReg
        .getContract()
        .then((badgeRegContract) => {
          return badgeRegUpdateFilter(badgeRegContract.address, [ [
            badgeRegContract.instance.Registered.signature,
            badgeRegContract.instance.Unregistered.signature,
            badgeRegContract.instance.MetaChanged.signature,
            badgeRegContract.instance.AddressChanged.signature
          ] ]);
        })
        .then(() => {
          shortFetchChanges();

          api.subscribe('eth_blockNumber', (err) => {
            if (err) {
              return;
            }

            fetchChanges();
          });
        });

      function onLogs (logs) {
        logs = contract.parseEventLogs(logs);
        logs.forEach((log) => {
          const certifier = certifiers.find((c) => c.address === log.address);

          if (!certifier) {
            throw new Error(`Could not find certifier at ${log.address}.`);
          }
          const { id, name, title, icon } = certifier;

          if (log.event === 'Revoked') {
            store.dispatch(removeCertification(log.params.who.value, id));
          } else {
            store.dispatch(addCertification(log.params.who.value, id, name, title, icon));
          }
        });
      }

      function onBadgeRegLogs (logs) {
        return badgeReg.getContract()
          .then((badgeRegContract) => {
            logs = badgeRegContract.parseEventLogs(logs);

            const ids = logs.map((log) => log.params && log.params.id.value.toNumber());

            return fetchCertifiers(uniq(ids));
          });
      }

      function _fetchChanges () {
        const method = filterChanged
          ? 'getFilterLogs'
          : 'getFilterChanges';

        filterChanged = false;

        api.eth[method](badgeRegFilter)
          .then(onBadgeRegLogs)
          .catch((err) => {
            console.error('Failed to fetch badge reg events:', err);
          })
          .then(() => api.eth[method](filter))
          .then(onLogs)
          .catch((err) => {
            console.error('Failed to fetch new certifier events:', err);
          });
      }

      const shortFetchChanges = debounce(_fetchChanges, 0.5 * 1000, { leading: true });
      const fetchChanges = debounce(shortFetchChanges, 10 * 1000, { leading: true });

      function fetchConfirmedEvents () {
        return updateFilter(certifiers.map((c) => c.address), [
          [ Confirmed.signature, Revoked.signature ],
          addresses
        ]).then(() => shortFetchChanges());
      }

      function fetchCertifiers (ids = []) {
        if (fetchCertifiersPromise) {
          return fetchCertifiersPromise;
        }

        let fetchEvents = false;

        const idsPromise = (certifiers.length === 0)
          ? badgeReg.certifierCount().then((count) => {
            return range(count);
          })
          : Promise.resolve(ids);

        fetchCertifiersPromise = idsPromise
          .then((ids) => {
            const promises = ids.map((id) => {
              return badgeReg.fetchCertifier(id)
                .then((cert) => {
                  if (!certifiers.some((c) => c.id === cert.id)) {
                    certifiers = certifiers.concat(cert);
                    fetchEvents = true;
                  }
                })
                .catch((err) => {
                  if (/does not exist/.test(err.toString())) {
                    return log.info(err.toString());
                  }

                  log.warn(`Could not fetch certifier ${id}:`, err);
                });
            });

            return Promise
              .all(promises)
              .then(() => {
                fetchCertifiersPromise = null;

                if (fetchEvents) {
                  return fetchConfirmedEvents();
                }
              });
          });

        return fetchCertifiersPromise;
      }

      return (next) => (action) => {
        switch (action.type) {
          case 'fetchCertifiers':
            fetchConfirmedEvents();

            break;
          case 'fetchCertifications':
            const { address } = action;

            if (!addresses.includes(address)) {
              addresses = addresses.concat(address);
              fetchConfirmedEvents();
            }

            break;
          case 'setVisibleAccounts':
            const _addresses = action.addresses || [];

            addresses = uniq(addresses.concat(_addresses));
            fetchConfirmedEvents();
            next(action);

            break;
          default:
            next(action);
        }
      };
    };
  }
}
