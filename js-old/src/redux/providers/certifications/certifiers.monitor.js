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

import { range } from 'lodash';

import { addCertification, removeCertification } from './actions';

import { getLogger, LOG_KEYS } from '~/config';
import Contract from '@parity/api/lib/contract';
import { bytesToHex, hexToAscii } from '@parity/api/lib/util/format';
import Contracts from '~/contracts';
import CertifierABI from '~/contracts/abi/certifier.json';
import { querier } from './enhanced-querier';

const log = getLogger(LOG_KEYS.CertificationsMiddleware);

let self = null;

export default class CertifiersMonitor {
  constructor (api, store) {
    this._api = api;
    this._name = 'Certifiers';
    this._store = store;

    this._contract = new Contract(this.api, CertifierABI);
    this._contractEvents = [ 'Confirmed', 'Revoked' ]
      .map((name) => this.contract.events.find((e) => e.name === name));

    this.certifiers = {};
    this.fetchedAccounts = {};

    this.load();
  }

  static get () {
    if (self) {
      return self;
    }

    self = new CertifiersMonitor();
    return self;
  }

  static init (api, store) {
    if (!self) {
      self = new CertifiersMonitor(api, store);
    }
  }

  get api () {
    return this._api;
  }

  get contract () {
    return this._contract;
  }

  get contractEvents () {
    return this._contractEvents;
  }

  get name () {
    return this._name;
  }

  get store () {
    return this._store;
  }

  get registry () {
    return this._registry;
  }

  get registryEvents () {
    return this._registryEvents;
  }

  checkFilters () {
    this.checkCertifiersFilter();
    this.checkRegistryFilter();
  }

  checkCertifiersFilter () {
    if (!this.certifiersFilter) {
      return;
    }

    this.api.eth.getFilterChanges(this.certifiersFilter)
      .then((logs) => {
        if (logs.length === 0) {
          return;
        }

        const parsedLogs = this.contract.parseEventLogs(logs).filter((log) => log.params);

        log.debug('received certifiers logs', parsedLogs);

        const promises = parsedLogs.map((log) => {
          const account = log.params.who.value;
          const certifier = Object.values(this.certifiers).find((c) => c.address === log.address);

          if (!certifier) {
            log.warn('could not find the certifier', { certifiers: this.certifiers, log });
            return Promise.resolve();
          }

          return this.fetchAccount(account, { ids: [ certifier.id ] });
        });

        return Promise.all(promises);
      })
      .catch((error) => {
        console.error(error);
      });
  }

  checkRegistryFilter () {
    if (!this.registryFilter) {
      return;
    }

    this.api.eth.getFilterChanges(this.registryFilter)
      .then((logs) => {
        if (logs.length === 0) {
          return;
        }

        const parsedLogs = this.contract.parseEventLogs(logs).filter((log) => log.params);
        const indexes = parsedLogs.map((log) => log.params && log.params.id.value.toNumber());

        log.debug('received registry logs', parsedLogs);
        return this.fetchElements(indexes);
      })
      .catch((error) => {
        console.error(error);
      });
  }

  /**
   * Initial load of the Monitor.
   * Fetch the contract from the Registry, and
   * load the elements addresses
   */
  load () {
    const badgeReg = Contracts.get().badgeReg;

    log.debug(`loading the ${this.name} monitor...`);
    return badgeReg.getContract()
      .then((registryContract) => {
        this._registry = registryContract;
        this._registryEvents = [ 'Registered', 'Unregistered', 'MetaChanged', 'AddressChanged' ]
          .map((name) => this.registry.events.find((e) => e.name === name));

        return this.registry.instance.badgeCount.call({});
      })
      .then((count) => {
        log.debug(`found ${count.toFormat()} registered contracts for ${this.name}`);
        return this.fetchElements(range(count.toNumber()));
      })
      .then(() => {
        return this.setRegistryFilter();
      })
      .then(() => {
        // Listen for new blocks
        return this.api.subscribe('eth_blockNumber', (err) => {
          if (err) {
            return;
          }

          this.checkFilters();
        });
      })
      .then(() => {
        log.debug(`loaded the ${this.name} monitor!`, this.certifiers);
      })
      .catch((error) => {
        log.error(error);
      });
  }

  /**
   * Fetch the given registered element
   */
  fetchElements (indexes) {
    const badgeReg = Contracts.get().badgeReg;
    const { instance } = this.registry;

    const sorted = indexes.sort();
    const from = sorted[0];
    const last = sorted[sorted.length - 1];
    const limit = last - from + 1;

    // Fetch the address, name and owner in one batch
    return querier(this.api, { address: instance.address, from, limit }, instance.badge)
      .then((results) => {
        const certifiers = results
          .map(([ address, name, owner ], index) => ({
            address, owner,
            id: index + from,
            name: hexToAscii(bytesToHex(name).replace(/(00)+$/, ''))
          }))
          .reduce((certifiers, certifier) => {
            const { id } = certifier;

            if (!/^(0x)?0+$/.test(certifier.address)) {
              certifiers[id] = certifier;
            } else if (certifiers[id]) {
              delete certifiers[id];
            }

            return certifiers;
          }, {});

        // Fetch the meta-data in serie
        return Object.values(certifiers).reduce((promise, certifier) => {
          return promise.then(() => badgeReg.fetchMeta(certifier.id))
            .then((meta) => {
              this.certifiers[certifier.id] = { ...certifier, ...meta };
            });
        }, Promise.resolve());
      })
      .then(() => log.debug('fetched certifiers', { certifiers: this.certifiers }))
      // Fetch the know accounts in case it's an update of the certifiers
      .then(() => this.fetchAccounts(Object.keys(this.fetchedAccounts), { ids: indexes, force: true }));
  }

  fetchAccounts (addresses, { ids = null, force = false } = {}) {
    const newAddresses = force
      ? addresses
      : addresses.filter((address) => !this.fetchedAccounts[address]);

    if (newAddresses.length === 0) {
      return Promise.resolve();
    }

    log.debug(`fetching values for "${addresses.join(' ; ')}" in ${this.name}...`);
    return newAddresses
      .reduce((promise, address) => {
        return promise.then(() => this.fetchAccount(address, { ids }));
      }, Promise.resolve())
      .then(() => {
        log.debug(`fetched values for "${addresses.join(' ; ')}" in ${this.name}!`);
      })
      .then(() => this.setCertifiersFilter());
  }

  fetchAccount (address, { ids = null } = {}) {
    let certifiers = Object.values(this.certifiers);

    // Only fetch values for the givens ids, if any
    if (ids) {
      certifiers = certifiers.filter((certifier) => ids.includes(certifier.id));
    }

    certifiers
      .reduce((promise, certifier) => {
        return promise
          .then(() => {
            return this.contract.at(certifier.address).instance.certified.call({}, [ address ]);
          })
          .then((certified) => {
            const { id, title, icon, name } = certifier;

            if (!certified) {
              return this.store.dispatch(removeCertification(address, id));
            }

            log.debug('seen as certified', { address, id, name, icon });
            this.store.dispatch(addCertification(address, id, name, title, icon));
          });
      }, Promise.resolve())
      .then(() => {
        this.fetchedAccounts[address] = true;
      });
  }

  setCertifiersFilter () {
    const accounts = Object.keys(this.fetchedAccounts);
    const addresses = Object.values(this.certifiers).map((c) => c.address);
    // The events have as first indexed data the account address
    const topics = [
      this.contractEvents.map((event) => '0x' + event.signature),
      accounts
    ];

    if (accounts.length === 0 || addresses.length === 0) {
      return;
    }

    const promise = this.certifiersFilter
      ? this.api.eth.uninstallFilter(this.certifiersFilter)
      : Promise.resolve();

    log.debug('setting up registry filter', { topics, accounts, addresses });

    return promise
      .then(() => this.api.eth.newFilter({
        fromBlock: 'latest',
        toBlock: 'latest',
        address: addresses,
        topics
      }))
      .then((filterId) => {
        this.certifiersFilter = filterId;
      })
      .catch((error) => {
        console.error(error);
      });
  }

  setRegistryFilter () {
    const { address } = this.registry.instance;
    const topics = [ this.registryEvents.map((event) => '0x' + event.signature) ];

    log.debug('setting up registry filter', { topics, address });

    return this.api.eth
      .newFilter({
        fromBlock: 'latest',
        toBlock: 'latest',
        address, topics
      })
      .then((filterId) => {
        this.registryFilter = filterId;
      })
      .catch((error) => {
        console.error(error);
      });
  }
}
