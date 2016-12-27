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

import { uniq, range } from 'lodash';
import debounce from 'debounce';

import CertifierABI from '~/contracts/abi/certifier.json';
import Contract from '~/api/contract';
import Contracts from '~/contracts';
import { addCertification, removeCertification } from './actions';

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
      let accounts = []; // these are addresses
      let filter = null;

      const onFilter = (filterId) => {
        filter = filterId;
        api.eth.getFilterLogs(filterId)
          .then(onLogs)
          .catch((err) => {
            console.error('Failed to fetch certifier events:', err);
          });
      };

      const onBadgeFilter = (filterId) => {
        api.eth.getFilterLogs(filterId)
          .then((logs) => {
            const ids = logs.map((log) => log.params.id.value.toNumber());
            return fetchCertifiers(uniq(ids));
          })
          .catch((err) => {
            console.error('Failed to fetch badgereg events:', err);
          });
      };

      const updateFilter = updatableFilter(api, onFilter);
      const badgeRegFilter = updatableFilter(api, onBadgeFilter);

      badgeReg
        .getContract()
        .then((badgeRegContract) => {
          badgeRegFilter(badgeRegContract.address, [
            badgeRegContract.instance.Registered.signature,
            badgeRegContract.instance.Unregistered.signature,
            badgeRegContract.instance.MetaChanged.signature,
            badgeRegContract.instance.AddressChanged.signature
          ]);
        });

      const onLogs = (logs) => {
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
      };

      const fetchChanges = debounce(() => {
        api.eth.getFilterChanges(filter)
          .then(onLogs)
          .catch((err) => {
            console.error('Failed to fetch new certifier events:', err);
          });
      }, 10 * 1000, true);

      api.subscribe('eth_blockNumber', (err) => {
        if (err) {
          return;
        }

        fetchChanges();
      });

      const fetchConfirmedEvents = () => {
        updateFilter(certifiers.map((c) => c.address), [
          [ Confirmed.signature, Revoked.signature ],
          accounts
        ]);
      };

      const fetchCertifiers = (ids) => {
        return ids.forEach((id) => {
          badgeReg.fetchCertifier(id)
            .then((cert) => {
              if (!certifiers.some((c) => c.id === cert.id)) {
                certifiers = certifiers.concat(cert);
                fetchConfirmedEvents();
              }
            })
            .catch((err) => {
              if (/does not exist/.test(err.toString())) {
                return console.warn(err.toString());
              }

              console.warn(`Could not fetch certifier ${id}:`, err);
            });
        });
      };

      return (next) => (action) => {
        switch (action.type) {
          case 'fetchCertifiers':
            // Initial fetch of certifiers
            if (certifiers.length === 0) {
              return badgeReg.certifierCount().then((count) => {
                fetchCertifiers(range(count));
              });
            }

            fetchConfirmedEvents();

            break;
          case 'fetchCertifications':
            const { address } = action;

            if (!accounts.includes(address)) {
              accounts = accounts.concat(address);
              fetchConfirmedEvents();
            }

            break;
          case 'setVisibleAccounts':
            const { addresses } = action;
            accounts = uniq(accounts.concat(addresses));
            fetchConfirmedEvents();

            break;
          default:
            next(action);
        }
      };
    };
  }
}
