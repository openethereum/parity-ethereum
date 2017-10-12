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

import { action, observable } from 'mobx';
import { uniq } from 'lodash';

import Contracts from '~/contracts';
import { vouchfor as vouchForAbi } from '~/contracts/abi';

let contractPromise = null;

export default class Store {
  @observable vouchers = [];

  constructor (api, app) {
    this._api = api;

    this.findVouchers(app);
  }

  async attachContract () {
    const address = await Contracts.get().registry.lookupAddress('vouchfor');

    if (!address || /^0x0*$/.test(address)) {
      return null;
    }

    const contract = await this._api.newContract(vouchForAbi, address);

    return contract;
  }

  async findVouchers ({ contentHash, id }) {
    if (!contentHash) {
      return;
    }

    if (!contractPromise) {
      contractPromise = this.attachContract();
    }

    const contract = await contractPromise;

    if (!contract) {
      return;
    }

    const vouchHash = await this.lookupHash(contract, `0x${contentHash}`);
    const vouchId = await this.lookupHash(contract, id);

    this.addVouchers(vouchHash, vouchId);
  }

  async lookupHash (contract, hash) {
    const vouchers = [];
    let lastItem = false;

    for (let index = 0; !lastItem; index++) {
      const voucher = await contract.instance.vouched.call({}, [hash, index]);

      if (/^0x0*$/.test(voucher)) {
        lastItem = true;
      } else {
        vouchers.push(voucher);
      }
    }

    return vouchers;
  }

  @action addVouchers = (vouchHash, vouchId) => {
    this.vouchers = uniq([].concat(this.vouchers.peek(), vouchHash, vouchId));
  }
}
