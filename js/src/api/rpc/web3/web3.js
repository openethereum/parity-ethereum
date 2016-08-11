import { inHex } from '../../format/input';

export default class Web3 {
  constructor (transport) {
    this._transport = transport;
  }

  clientVersion () {
    return this._transport
      .execute('web3_clientVersion');
  }

  sha3 (hexStr) {
    return this._transport
      .execute('web3_sha3', inHex(hexStr));
  }
}
