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

export default class Personal {
  constructor (updateSubscriptions, api, subscriber) {
    this._subscriber = subscriber;
    this._api = api;
    this._updateSubscriptions = updateSubscriptions;
    this._started = false;
  }

  get isStarted () {
    return this._started;
  }

  start () {
    this._started = true;

    return Promise.all([
      this._listAccounts(),
      this._accountsInfo(),
      this._loggingSubscribe()
    ]);
  }

  _listAccounts = () => {
    return this._api.personal
      .listAccounts()
      .then((accounts) => {
        this._updateSubscriptions('personal_listAccounts', null, accounts);
      });
  }

  _accountsInfo = () => {
    return this._api.personal
      .accountsInfo()
      .then((info) => {
        this._updateSubscriptions('personal_accountsInfo', null, info);
      });
  }

  _loggingSubscribe () {
    return this._subscriber.subscribe('logging', (error, data) => {
      if (error || !data) {
        return;
      }

      switch (data.method) {
        case 'personal_importGethAccounts':
        case 'personal_newAccount':
        case 'personal_newAccountFromPhrase':
        case 'personal_newAccountFromWallet':
          this._listAccounts();
          this._accountsInfo();
          return;

        case 'personal_setAccountName':
        case 'personal_setAccountMeta':
          this._accountsInfo();
          return;
      }
    });
  }
}
