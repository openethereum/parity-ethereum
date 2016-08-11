export default class BytesTaken {
  constructor (bytes, newOffset) {
    this._bytes = bytes;
    this._newOffset = newOffset;
  }

  get bytes () {
    return this._bytes;
  }

  get newOffset () {
    return this._newOffset;
  }
}
