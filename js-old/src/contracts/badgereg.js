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

import { bytesToHex, hexToAscii } from '~/api/util/format';

import ABI from './abi/certifier.json';

const ZERO20 = '0x0000000000000000000000000000000000000000';
const ZERO32 = '0x0000000000000000000000000000000000000000000000000000000000000000';

export default class BadgeReg {
  constructor (api, registry) {
    this._api = api;
    this._registry = registry;

    registry.getContract('badgereg');
    this.certifiers = []; // by id
    this.contracts = {}; // by name
  }

  getContract () {
    return this._registry.getContract('badgereg');
  }

  certifierCount () {
    return this
      .getContract()
      .then((badgeReg) => {
        return badgeReg.instance.badgeCount.call({}, [])
          .then((count) => count.valueOf());
      });
  }

  fetchCertifier (id) {
    if (this.certifiers[id]) {
      return Promise.resolve(this.certifiers[id]);
    }

    return this
      .getContract()
      .then((badgeReg) => {
        return badgeReg.instance.badge.call({}, [ id ]);
      })
      .then(([ address, name ]) => {
        if (address === ZERO20) {
          throw new Error(`Certifier ${id} does not exist.`);
        }

        name = bytesToHex(name);
        name = name === ZERO32
          ? null
          : hexToAscii(name);

        return this.fetchMeta(id)
          .then(({ title, icon }) => {
            const data = { address, id, name, title, icon };

            this.certifiers[id] = data;
            return data;
          });
      });
  }

  fetchCertifierByName (name) {
    return this
      .getContract()
      .then((badgeReg) => {
        return badgeReg.instance.fromName.call({}, [ name ]);
      })
      .then(([ id, address, owner ]) => {
        if (address === ZERO20) {
          throw new Error(`Certifier ${name} does not exist.`);
        }

        return this.fetchMeta(id)
          .then(({ title, icon }) => {
            const data = { address, id, name, title, icon };

            this.certifiers[id] = data;
            return data;
          });
      });
  }

  fetchMeta (id) {
    return this
      .getContract()
      .then((badgeReg) => {
        return Promise.all([
          badgeReg.instance.meta.call({}, [id, 'TITLE']),
          badgeReg.instance.meta.call({}, [id, 'IMG'])
        ]);
      })
      .then(([ title, icon ]) => {
        title = bytesToHex(title);
        title = title === ZERO32 ? null : hexToAscii(title);

        if (bytesToHex(icon) === ZERO32) {
          icon = null;
        }

        return { title, icon };
      });
  }

  checkIfCertified (certifier, address) {
    if (!this.contracts[certifier]) {
      this.contracts[certifier] = this._api.newContract(ABI, certifier);
    }

    const contract = this.contracts[certifier];

    return contract.instance.certified.call({}, [address]);
  }
}
