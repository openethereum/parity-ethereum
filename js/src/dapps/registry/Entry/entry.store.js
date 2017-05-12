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

import ApplicationStore from '../Application/application.store';

import { checkOwnerReverse } from '../util/registry';
import { postTransaction } from '../util/transactions';

export default class Entry {
  @observable dropping = false;
  @observable reversed = false;
  @observable reversing = false;

  applicationStore = ApplicationStore.get();

  isOwner = null;
  name = null;
  owner = null;
  address = null;
  image = null;
  content = null;

  constructor ({ name, owner, address, image, content, ownerReverseName }) {
    const { accounts } = this.applicationStore;

    this.name = name;
    this.owner = owner;
    this.address = address;
    this.image = image;
    this.content = content;

    if (owner) {
      const lcOwner = owner.toLowerCase();
      const isOwner = !!accounts
        .find((account) => account.address.toLowerCase() === lcOwner);

      this.isOwner = isOwner;
    }

    if (ownerReverseName === this.name) {
      this.reversed = true;
    }
  }

  checkOwnerReverse () {
    const { contract } = this.applicationStore;

    return checkOwnerReverse(contract, this.owner);
  }

  drop () {
    const { api, contract } = this.applicationStore;

    const method = contract.instance.drop;
    const options = { from: this.owner };
    const values = [ api.util.sha3.text(this.name.toLowerCase()) ];

    this.setDropping(true);
    return postTransaction(api, method, options, values)
      .catch((error) => {
        console.error('dropping', error);
      })
      .then(() => {
        this.setDropping(false);
      });
  }

  modifyMetadata (key, newValue) {
    const { api, contract } = this.applicationStore;

    const isAddress = key === 'A';
    const method = isAddress
      ? contract.instance.setAddress
      : contract.instance.setData;

    const options = { from: this.owner };
    const values = [ api.util.sha3.text(this.name.toLowerCase()), key, newValue ];

    return postTransaction(api, method, options, values);
  }

  modifyOwner (newOwner) {
    const { api, contract } = this.applicationStore;

    const method = contract.instance.transfer;
    const options = { from: this.owner };
    const values = [ api.util.sha3.text(this.name.toLowerCase()), newOwner ];

    return postTransaction(api, method, options, values);
  }

  reverse () {
    const { api, contract } = this.applicationStore;

    const reverseMethod = contract.instance.proposeReverse;
    const reverseValues = [ this.name.toLowerCase(), this.owner ];

    this.setReversing(true);
    return postTransaction(api, reverseMethod, {}, reverseValues)
      .then(() => {
        const confirmMethod = contract.instance.confirmReverse;
        const confirmOptions = { from: this.owner };
        const confirmValues = [ this.name.toLowerCase() ];

        return postTransaction(api, confirmMethod, confirmOptions, confirmValues);
      })
      .catch((error) => {
        console.error('reversing', error);
      })
      .then(() => {
        this.setReversing(false);
      });
  }

  @action
  setDropping (dropping) {
    this.dropping = dropping;
  }

  @action
  setReversing (reversing) {
    this.reversing = reversing;
  }
}
