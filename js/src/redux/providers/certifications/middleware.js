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
// along with Parity.  If not, see <http://www.gnu.org/licenses/>.

import { uniq } from 'lodash';

import ABI from '~/contracts/abi/certifier.json';
import Contract from '~/api/contract';
import Contracts from '~/contracts';
import { addCertification } from './actions';

export default class CertificationsMiddleware {
  toMiddleware () {
    const api = Contracts.get()._api;
    const badgeReg = Contracts.get().badgeReg;
    const contract = new Contract(api, ABI);
    const Confirmed = contract.events.find((e) => e.name === 'Confirmed');

    let certifiers = [];
    let accounts = []; // these are addresses

    const fetchConfirmedEvents = (dispatch) => {
      if (certifiers.length === 0 || accounts.length === 0) return;
      api.eth.getLogs({
        fromBlock: 0,
        toBlock: 'latest',
        address: certifiers.map((c) => c.address),
        topics: [ Confirmed.signature, accounts ]
      })
        .then((logs) => contract.parseEventLogs(logs))
        .then((logs) => {
          logs.forEach((log) => {
            const certifier = certifiers.find((c) => c.address === log.address);
            if (!certifier) throw new Error(`Could not find certifier at ${log.address}.`);
            const { id, name, title, icon } = certifier;
            dispatch(addCertification(log.params.who.value, id, name, title, icon));
          });
        })
        .catch((err) => {
          console.error('Failed to fetch Confirmed events:', err);
        });
    };

    return (store) => (next) => (action) => {
      if (action.type === 'fetchCertifiers') {
        badgeReg.nrOfCertifiers().then((count) => {
          new Array(+count).fill(null).forEach((_, id) => {
            badgeReg.fetchCertifier(id)
              .then((cert) => {
                if (!certifiers.some((c) => c.id === cert.id)) {
                  certifiers = certifiers.concat(cert);
                  fetchConfirmedEvents(store.dispatch);
                }
              })
              .catch((err) => {
                console.warn(`Could not fetch certifier ${id}:`, err);
              });
          });
        });
      } else if (action.type === 'fetchCertifications') {
        const { address } = action;

        if (!accounts.includes(address)) {
          accounts = accounts.concat(address);
          fetchConfirmedEvents(store.dispatch);
        }
      } else if (action.type === 'setVisibleAccounts') {
        const { addresses } = action;
        accounts = uniq(accounts.concat(addresses));
        fetchConfirmedEvents(store.dispatch);
      } else return next(action);
    };
  }
}
