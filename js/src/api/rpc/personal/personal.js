import { inAddress, inNumber10, inOptions } from '../../format/input';
import { outAccountInfo, outAddress } from '../../format/output';

export default class Personal {
  constructor (transport) {
    this._transport = transport;
  }

  accountsInfo () {
    return this._transport
      .execute('personal_accountsInfo')
      .then(outAccountInfo);
  }

  listAccounts () {
    return this._transport
      .execute('personal_listAccounts')
      .then((accounts) => (accounts || []).map(outAddress));
  }

  newAccount (password) {
    return this._transport
      .execute('personal_newAccount', password)
      .then(outAddress);
  }

  newAccountFromPhrase (phrase, password) {
    return this._transport
      .execute('personal_newAccountFromPhrase', phrase, password)
      .then(outAddress);
  }

  newAccountFromWallet (json, password) {
    return this._transport
      .execute('personal_newAccountFromWallet', json, password)
      .then(outAddress);
  }

  setAccountName (address, name) {
    return this._transport
      .execute('personal_setAccountName', inAddress(address), name);
  }

  setAccountMeta (address, meta) {
    return this._transport
      .execute('personal_setAccountMeta', inAddress(address), JSON.stringify(meta));
  }

  signAndSendTransaction (options, password) {
    return this._transport
      .execute('personal_signAndSendTransaction', inOptions(options), password);
  }

  signerEnabled () {
    return this._transport
      .execute('personal_signerEnabled');
  }

  unlockAccount (account, password, duration = 1) {
    return this._transport
      .execute('personal_unlockAccount', inAddress(account), password, inNumber10(duration));
  }
}
