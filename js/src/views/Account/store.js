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
import FileSaver from 'file-saver';

export default class Store {
  @observable isDeleteVisible = false;
  @observable isEditVisible = false;
  @observable isExportVisible = false;
  @observable isFaucetVisible = false;
  @observable isFundVisible = false;
  @observable isPasswordVisible = false;
  @observable isTransferVisible = false;
  @observable isVerificationVisible = false;
  @observable exportValue = '';

  insertProps (api, accounts, address, newError) {
    this._api = api;
    this._accounts = accounts;
    this._address = address;
    this._newError = newError;
  }

  @action editExportValue = (event, value) => {
    this.exportValue = value;
  }

  @action toggleDeleteDialog = () => {
    this.isDeleteVisible = !this.isDeleteVisible;
  }

  @action toggleEditDialog = () => {
    this.isEditVisible = !this.isEditVisible;
  }

  @action toggleExportDialog = () => {
    this.isExportVisible = !this.isExportVisible;
  }

  @action toggleFaucetDialog = () => {
    this.isFaucetVisible = !this.isFaucetVisible;
  }

  @action toggleFundDialog = () => {
    this.isFundVisible = !this.isFundVisible;
  }

  @action togglePasswordDialog = () => {
    this.isPasswordVisible = !this.isPasswordVisible;
  }

  @action toggleTransferDialog = () => {
    this.isTransferVisible = !this.isTransferVisible;
  }

  @action toggleVerificationDialog = () => {
    this.isVerificationVisible = !this.isVerificationVisible;
  }

  onExport = () => {
    const { parity } = this._api;

    parity.exportAccount(this._address, this.exportValue)
      .then((content) => {
        const text = JSON.stringify(content, null, 4);
        const blob = new Blob([ text ], { type: 'application/json' });
        const filename = this._accounts[this._address].uuid;

        FileSaver.saveAs(blob, `${filename}.json`);
        setTimeout(() => {
          this.toggleExportDialog();
        }, 500);
      })
      .catch((err) => {
        const { passwordHint } = this._accounts[this._address].meta;

        this._newError({
          message: `[${err.code}] - Incorrect password. Password Hint: (${passwordHint})`
        });
      });
  }
}
