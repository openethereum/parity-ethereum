export default class Personal {
  constructor (updateSubscriptions, api, subscriber) {
    this._subscriber = subscriber;
    this._api = api;
    this._updateSubscriptions = updateSubscriptions;
    this._started = false;

    this._lastAccounts = [];
    this._lastInfo = {};
  }

  get isStarted () {
    return this._started;
  }

  start () {
    this._started = true;

    this._listAccounts();
    this._accountsInfo();
    this._loggingSubscribe();
  }

  _listAccounts = () => {
    this._api.personal
      .listAccounts()
      .then((accounts) => {
        let different = false;

        if (accounts.length !== this._lastAccounts.length) {
          different = true;
        }

        if (different) {
          this._lastAccounts = accounts;
          this._updateSubscriptions('personal_listAccounts', null, accounts);
        }
      });
  }

  _accountsInfo = () => {
    this._api.personal
      .accountsInfo()
      .then((info) => {
        const infoKeys = Object.keys(info);
        const lastKeys = Object.keys(this._lastInfo);
        let different = false;

        if (infoKeys.length !== lastKeys.length) {
          different = true;
        } else {
          different = !!infoKeys.find((key) => {
            return (!lastKeys[key] || this._lastInfo[key].name !== info[key].name);
          });
        }

        if (different) {
          this._lastInfo = info;
          this._updateSubscriptions('personal_accountsInfo', null, info);
        }
      });
  }

  _loggingSubscribe () {
    this._subscriber.subscribe('logging', (error, data) => {
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
