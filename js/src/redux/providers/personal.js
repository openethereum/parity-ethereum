import { personalAccountsInfo } from './personalActions';

export default class Personal {
  constructor (store, api) {
    this._api = api;
    this._store = store;
  }

  start () {
    this._subscribeAccountsInfo();
  }

  _subscribeAccountsInfo () {
    this._api.subscribe('personal_accountsInfo', (error, accountsInfo) => {
      if (error) {
        console.error('personal_accountsInfo', error);
        return;
      }

      this._store.dispatch(personalAccountsInfo(accountsInfo));
    });
  }
}
