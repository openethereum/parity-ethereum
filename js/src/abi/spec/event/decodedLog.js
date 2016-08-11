export default class DecodedLog {
  constructor (params, address) {
    this._params = params;
    this._address = address;
  }

  get address () {
    return this._address;
  }

  get params () {
    return this._params;
  }
}
