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

import { action, computed, observable, toJS } from 'mobx';
import React from 'react';

import { showSnackbar } from '~/redux/providers/snackbarActions';

export default class Store {
  @observable address = null;
  @observable meta = null;
  @observable newPassword = '';
  @observable newPasswordHint = '';
  @observable newPasswordRepeat = '';
  @observable password = '';
  @observable passwordHint = '';

  constructor (api, account) {
    const { address, meta } = account;

    this._api = api;

    this.address = address;
    this.meta = meta || {};
    this.passwordHint = this.meta.passwordHint || '';
  }

  @computed isRepeatValid () {
    return this.newPasswordRepeat === this.newPassword;
  }

  @action setPassword = (password) => {
    this.password = password;
  }

  @action setNewPassword = (password) => {
    this.newPassword = password;
  }

  @action setNewPasswordHint = (passwordHint) => {
    this.newPasswordHint = passwordHint;
  }

  @action setNewPasswordRepeat = (password) => {
    this.newPasswordRepeat = password;
  }

  changePassword = () => {
    const { onClose } = this.props;
    const { currentPass } = this.state;

    if (this.newPasswordRepeat !== this.newPassword) {
      return;
    }

    this.setState({ waiting: true, showMessage: false });

    return this._api.parity
      .testPassword(this.address, currentPass)
      .then(correct => {
        if (!correct) {
          const message = {
            value: 'This provided current password is not correct',
            success: false
          };

          this.setState({ waiting: false, message, showMessage: true });

          return false;
        }

        const meta = Object.assign({}, toJS(this.meta), {
          passwordHint: this.newPasswordHint
        });

        return Promise
          .all([
            this._api.parity.setAccountMeta(this.address, meta),
            this._api.parity.changePassword(this.address, this.password, this.newPassword)
          ])
          .then(() => {
            showSnackbar(<div>Your password has been successfully changed.</div>);
            this.setState({ waiting: false, showMessage: false });
            onClose();
          });
      })
      .catch((error) => {
        console.error('changePassword', error);
        this.setState({ waiting: false });
      });
  }

  testPassword = () => {
    this.setState({ waiting: true, showMessage: false });

    return this._api.parity
      .testPassword(this.address, this.password)
      .then(correct => {
        const message = correct
          ? { value: 'This password is correct', success: true }
          : { value: 'This password is not correct', success: false };

        this.setState({ waiting: false, message, showMessage: true });
      })
      .catch((error) => {
        console.error('testPassword', error);
        this.setState({ waiting: false });
      });
  }
}
