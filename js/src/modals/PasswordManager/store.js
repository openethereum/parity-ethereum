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

import { observable } from 'mobx';
import React from 'react';

import { showSnackbar } from '~/redux/providers/snackbarActions';

export default class Store {
  @observable address = null;
  @observable meta = null;

  constructor (api, account) {
    const { address, meta } = account;

    this._api = api;

    this.address = address;
    this.meta = meta;
  }

  changePassword = () => {
    const { account, onClose } = this.props;
    const { currentPass, newPass, repeatNewPass, passwordHint } = this.state;

    if (repeatNewPass !== newPass) {
      return;
    }

    this.setState({ waiting: true, showMessage: false });

    return this._api.parity
      .testPassword(account.address, currentPass)
      .then(correct => {
        if (!correct) {
          const message = {
            value: 'This provided current password is not correct',
            success: false
          };

          this.setState({ waiting: false, message, showMessage: true });

          return false;
        }

        const meta = Object.assign({}, account.meta, {
          passwordHint
        });

        return Promise
          .all([
            this._api.parity.setAccountMeta(account.address, meta),
            this._api.parity.changePassword(account.address, currentPass, newPass)
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
    const { account } = this.props;
    const { currentPass } = this.state;

    this.setState({ waiting: true, showMessage: false });

    return this._api.parity
      .testPassword(account.address, currentPass)
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
