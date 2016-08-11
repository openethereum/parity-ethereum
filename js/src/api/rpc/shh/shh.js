export default class Personal {
  constructor (transport) {
    this._transport = transport;
  }

  addToGroup (identity) {
    return this._transport
      .execute('shh_addToGroup', identity);
  }

  getFilterChanges (filterId) {
    return this._transport
      .execute('shh_getFilterChanges', filterId);
  }

  getMessages (filterId) {
    return this._transport
      .execute('shh_getMessages', filterId);
  }

  hasIdentity (identity) {
    return this._transport
      .execute('shh_hasIdentity', identity);
  }

  newFilter (options) {
    return this._transport
      .execute('shh_newFilter', options);
  }

  newGroup () {
    return this._transport
      .execute('shh_newGroup');
  }

  newIdentity () {
    return this._transport
      .execute('shh_newIdentity');
  }

  post (options) {
    return this._transport
      .execute('shh_post', options);
  }

  uninstallFilter (filterId) {
    return this._transport
      .execute('shh_uninstallFilter', filterId);
  }

  version () {
    return this._transport
      .execute('shh_version');
  }
}
