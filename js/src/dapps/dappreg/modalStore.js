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

import { action, observable, transaction } from 'mobx';

import { trackRequest } from './parity';

let instance = null;

export default class ModalStore {
  @observable errorDelete = null;
  @observable errorRegister = null;
  @observable errorUpdate = null;
  @observable stepDelete = 0;
  @observable stepRegister = 0;
  @observable stepUpdate = 0;
  @observable showingDelete = false;
  @observable showingRegister = false;
  @observable showingUpdate = false;
  @observable showingWarning = true;

  @observable dappId = null;
  @observable updates = {};

  _dappsStore = null;

  constructor (dappsStore) {
    this._dappsStore = dappsStore;
  }

  static instance (dappsStore) {
    if (!instance) {
      instance = new ModalStore(dappsStore);
    }

    return instance;
  }

  @action setDeleteError (error) {
    transaction(() => {
      this.setDeleteStep(0);
      this.errorDelete = error;
    });
  }

  @action setDeleteStep (step) {
    this.stepDelete = step;
  }

  @action showDelete () {
    transaction(() => {
      this.setDeleteStep(1);
      this.errorDelete = null;
      this.showingDelete = true;
    });
  }

  @action hideDelete () {
    this.showingDelete = false;
  }

  @action setRegisterError (error) {
    transaction(() => {
      this.setRegisterStep(0);
      this.errorRegister = error;
    });
  }

  @action setRegisterStep (step) {
    this.stepRegister = step;
  }

  @action showRegister () {
    transaction(() => {
      this.setRegisterStep(1);
      this.errorRegister = null;
      this.showingRegister = true;
    });
  }

  @action hideRegister () {
    transaction(() => {
      this.showingRegister = false;
    });
  }

  @action setUpdateError (error) {
    transaction(() => {
      this.setUpdateStep(0);
      this.errorUpdate = error;
    });
  }

  @action setUpdateStep (step) {
    this.stepUpdate = step;
  }

  @action showUpdate () {
    transaction(() => {
      this.setUpdateStep(1);
      this.errorUpdate = null;
      this.showingUpdate = true;
    });
  }

  @action hideUpdate () {
    transaction(() => {
      this.showingUpdate = false;
    });
  }

  @action hideWarning () {
    this.showingWarning = false;
  }

  @action handleDelete (dapp) {
    const { id, owner } = dapp;

    this.dappId = id;
    this.dappOwner = owner.address;
    return this.showDelete();
  }

  @action handleUpdate (data) {
    const { id, owner, updates } = data;

    this.dappId = id;
    this.dappOwner = owner.address;
    this.updates = updates;
    return this.showUpdate();
  }

  doDelete () {
    const { dappId, ownerAddress } = this;

    this.setDeleteStep(2);

    const values = [ dappId ];
    const options = {
      from: ownerAddress
    };

    console.log('ModalStore:doDelete', `performing deletion for ${dappId} from ${options.from}`);

    this._dappsStore._instanceReg
      .unregister.estimateGas(options, values)
      .then((gas) => {
        const newGas = gas.mul(1.2);

        console.log('ModalStore:doDelete', `gas estimated as ${gas.toFormat(0)}, setting to ${newGas.toFormat(0)}`);

        options.gas = newGas.toFixed(0);

        const request = this._dappsStore._instanceReg.unregister.postTransaction(options, values);
        const statusCallback = (error, status) => {
          if (error) {
            return console.error('ModalStore::doDelete::statusCallback', error);
          }

          if (status.transactionHash) {
            return this.setDeleteStep(3);
          }

          if (status.transactionReceipt) {
            this.setDeleteStep(4);
            return this._dappsStore.removeApp(dappId);
          }
        };

        return trackRequest(request, statusCallback);
      })
      .catch((error) => {
        console.error('ModalStore::doDelete', error);
        this.setDeleteError(error);
      });
  }

  doRegister () {
    this.setRegisterStep(2);

    const appId = this._dappsStore.wipApp.id;
    const values = [appId];
    const options = {
      from: this._dappsStore.currentAccount.address,
      value: this._dappsStore.fee
    };

    console.log('ModalStore:doRegister', `performing registration for ${appId} from ${this._dappsStore.currentAccount.address}`);

    this._dappsStore._instanceReg
      .register.estimateGas(options, values)
      .then((gas) => {
        const newGas = gas.mul(1.2);

        console.log('ModalStore:doRegister', `gas estimated as ${gas.toFormat(0)}, setting to ${newGas.toFormat(0)}`);

        options.gas = newGas.toFixed(0);

        const request = this._dappsStore._instanceReg.register.postTransaction(options, values);
        const statusCallback = (error, status) => {
          if (error) {
          } else if (status.signerRequestId) {
          } else if (status.transactionHash) {
            this.setRegisterStep(3);
          } else if (status.transactionReceipt) {
            this.setRegisterStep(4);
            this._dappsStore.addApp(appId, this._dappsStore.currentAccount);
          }
        };

        return trackRequest(request, statusCallback);
      })
      .catch((error) => {
        console.error('ModalStore:doRegister', error);
        this.setRegisterError(error);
      });
  }

  doUpdate () {
    const { dappId, ownerAddress, updates } = this;

    this.setUpdateStep(2);

    const statusCallback = (error, status) => {
      if (error) {
      } else if (status.signerRequestId) {
      } else if (status.transactionHash) {
        this.setUpdateStep(3);
      } else if (status.transactionReceipt) {
        this.setUpdateStep(4);
      }
    };

    const options = {
      from: ownerAddress
    };
    const types = {
      'content': 'CONTENT',
      'image': 'IMG',
      'manifest': 'MANIFEST'
    };
    const promises = Object
      .keys(types)
      .filter((type) => updates[type])
      .map((type) => {
        const values = [ dappId, types[type], updates[type] ];

        return this._dappsStore._instanceReg.setMeta
          .estimateGas(options, values)
          .then((gas) => {
            const nextGas = gas.map((gas) => gas.mul(1.2));
            const nextOptions = { ...options, gas: nextGas.toFixed(0) };

            console.log('ModalStore:doUpdate', `performing updates for ${dappId}`, nextOptions, values);

            return this._dappsStore._instanceReg.setMeta.postTransaction(nextOptions, values);
          })
          .then((request) => {
            return trackRequest(request, statusCallback);
          });
      });

    return Promise.all(promises)
      .then((requests) => {

      })
      .then(() => {
        this._dappsStore.refreshApp(dappId);
      })
      .catch((error) => {
        console.error('ModalStore:doUpdate', error);
        this.setUpdateError(error);
      });
  }
}
