export default class DecodeResult {
  constructor (token, newOffset) {
    this._token = token;
    this._newOffset = newOffset;
  }

  get token () {
    return this._token;
  }

  get newOffset () {
    return this._newOffset;
  }
}
