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

import { action, computed, observable, transaction } from 'mobx';

export default class Store {
  @observable activeTab = 0;
  @observable address = null;
  @observable busy = false;
  @observable infoMessage = null;
  @observable meta = null;
  @observable newPassword = '';
  @observable newPasswordHint = '';
  @observable newPasswordRepeat = '';
  @observable password = '';
  @observable passwordHint = '';
  @observable validatePassword = '';

  constructor (api, account) {
    this._api = api;

    this.address = account.address;
    this.meta = account.meta || {};
    this.passwordHint = this.meta.passwordHint || '';
  }

  @computed get isRepeatValid () {
    return this.newPasswordRepeat === this.newPassword;
  }

  @action setActiveTab = (activeTab) => {
    transaction(() => {
      this.activeTab = activeTab;
      this.setInfoMessage();
    });
  }

  @action setBusy = (busy, message) => {
    transaction(() => {
      this.busy = busy;
      this.setInfoMessage(message);
    });
  }

  @action setInfoMessage = (message = null) => {
    this.infoMessage = message;
  }

  @action setPassword = (password) => {
    transaction(() => {
      this.password = password;
      this.setInfoMessage();
    });
  }

  @action setNewPassword = (password) => {
    transaction(() => {
      this.newPassword = password;
      this.setInfoMessage();
    });
  }

  @action setNewPasswordHint = (passwordHint) => {
    transaction(() => {
      this.newPasswordHint = passwordHint;
      this.setInfoMessage();
    });
  }

  @action setNewPasswordRepeat = (password) => {
    transaction(() => {
      this.newPasswordRepeat = password;
      this.setInfoMessage();
    });
  }

  @action setValidatePassword = (password) => {
    transaction(() => {
      this.validatePassword = password;
      this.setInfoMessage();
    });
  }

  changePassword = () => {
    if (!this.isRepeatValid) {
      return Promise.resolve(false);
    }

    this.setBusy(true);

    return this
      .testPassword(this.password)
      .then((result) => {
        if (!result) {
          return false;
        }

        const meta = Object.assign({}, this.meta, {
          passwordHint: this.newPasswordHint
        });

        return Promise
          .all([
            this._api.parity.setAccountMeta(this.address, meta),
            this._api.parity.changePassword(this.address, this.password, this.newPassword)
          ])
          .then(() => {
            this.setBusy(false);
            return true;
          });
      })
      .catch((error) => {
        console.error('changePassword', error);
        this.setBusy(false);
        throw error;
      });
  }

  testPassword = (password) => {
    this.setBusy(true);

    return this._api.parity
      .testPassword(this.address, password || this.validatePassword)
      .then((success) => {
        this.setBusy(false, {
          success,
          value: success
            ? 'This password is correct'
            : 'This password is not correct'
        });

        return success;
      })
      .catch((error) => {
        console.error('testPassword', error);
        this.setBusy(false);
        throw error;
      });
  }
}
