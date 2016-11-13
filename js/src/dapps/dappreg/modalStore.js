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

import { action, observable, transaction } from 'mobx';

import { api } from './parity';
import DappsStore from './dappsStore';

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

  _dappsStore = DappsStore.instance();

  static instance () {
    if (!instance) {
      instance = new ModalStore();
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
      this.showingRegister = true;
    });
  }

  @action hideRegister () {
    transaction(() => {
      this._dappsStore.setEditing(false);
      this._dappsStore.setNew(false);
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
      this.showingUpdate = true;
    });
  }

  @action hideUpdate () {
    transaction(() => {
      this._dappsStore.setEditing(false);
      this._dappsStore.setNew(false);
      this.showingUpdate = false;
    });
  }

  doRegister () {
    this.setRegisterStep(2);

    const values = [this._dappsStore.wipApp.id];
    const options = {
      from: this._dappsStore.currentAccount.address,
      value: this._dappsStore.fee
    };

    console.log('ModalStore:doRegister', `performing registration for ${this._dappsStore.wipApp.id} from ${this._dappsStore.currentAccount.address}`);

    this._dappsStore._instanceReg
      .register.estimateGas(options, values)
      .then((gas) => {
        const newGas = gas.mul(1.2);

        console.log('ModalStore:doRegister', `gas estimated as ${gas.toFormat(0)}, setting to ${newGas.toFormat(0)}`);

        options.gas = newGas.toFixed(0);
        return this._dappsStore._instanceReg.register.postTransaction(options, values);
      })
      .then((requestId) => {
        console.log('ModalStore:doRegister', `posted to signer with requestId ${requestId.toString()}`);
        return api.pollMethod('parity_checkRequest', requestId);
      })
      .catch((error) => {
        console.error('ModalStore:doRegister', error);
        this.setRegisterError(error);
      });
  }
}
