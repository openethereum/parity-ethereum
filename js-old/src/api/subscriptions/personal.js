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

export default class Personal {
  constructor (updateSubscriptions, api, subscriber) {
    this._subscriber = subscriber;
    this._api = api;
    this._updateSubscriptions = updateSubscriptions;
    this._started = false;

    this._lastDefaultAccount = '0x0';
    this._pollTimerId = null;
  }

  get isStarted () {
    return this._started;
  }

  start () {
    this._started = true;

    return Promise.all([
      this._defaultAccount(),
      this._listAccounts(),
      this._accountsInfo(),
      this._loggingSubscribe()
    ]);
  }

  // FIXME: Because of the different API instances, the "wait for valid changes" approach
  // doesn't work. Since the defaultAccount is critical to operation, we poll in exactly
  // same way we do in ../eth (ala eth_blockNumber) and update. This should be moved
  // to pub-sub as it becomes available
  _defaultAccount = (timerDisabled = false) => {
    const nextTimeout = (timeout = 1000) => {
      if (!timerDisabled) {
        this._pollTimerId = setTimeout(() => {
          this._defaultAccount();
        }, timeout);
      }
    };

    if (!this._api.transport.isConnected) {
      nextTimeout(500);
      return;
    }

    return this._api.parity
      .defaultAccount()
      .then((defaultAccount) => {
        if (this._lastDefaultAccount !== defaultAccount) {
          this._lastDefaultAccount = defaultAccount;
          this._updateSubscriptions('parity_defaultAccount', null, defaultAccount);
        }

        nextTimeout();
      })
      .catch(() => nextTimeout());
  }

  _listAccounts = () => {
    return this._api.eth
      .accounts()
      .then((accounts) => {
        this._updateSubscriptions('eth_accounts', null, accounts);
      });
  }

  _accountsInfo = () => {
    return this._api.parity
      .accountsInfo()
      .then((info) => {
        this._updateSubscriptions('parity_accountsInfo', null, info);

        return this._api.parity
          .allAccountsInfo()
          .catch(() => {
            // NOTE: This fails on non-secure APIs, swallow error
            return {};
          })
          .then((allInfo) => {
            this._updateSubscriptions('parity_allAccountsInfo', null, allInfo);
          });
      });
  }

  _loggingSubscribe () {
    return this._subscriber.subscribe('logging', (error, data) => {
      if (error || !data) {
        return;
      }

      switch (data.method) {
        case 'parity_closeVault':
        case 'parity_openVault':
        case 'parity_killAccount':
        case 'parity_importGethAccounts':
        case 'parity_newAccountFromPhrase':
        case 'parity_newAccountFromWallet':
        case 'personal_newAccount':
          this._defaultAccount(true);
          this._listAccounts();
          this._accountsInfo();
          return;

        case 'parity_removeAddress':
        case 'parity_setAccountName':
        case 'parity_setAccountMeta':
          this._accountsInfo();
          return;

        case 'parity_setDappAddresses':
        case 'parity_setDappDefaultAddress':
        case 'parity_setNewDappsAddresses':
        case 'parity_setNewDappsDefaultAddress':
          this._defaultAccount(true);
          this._listAccounts();
          return;
      }
    });
  }
}
