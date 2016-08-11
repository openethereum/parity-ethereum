import { outNumber } from '../../format/output';

export default class Net {
  constructor (transport) {
    this._transport = transport;
  }

  listening () {
    return this._transport
      .execute('net_listening');
  }

  peerCount () {
    return this._transport
      .execute('net_peerCount')
      .then(outNumber);
  }

  version () {
    return this._transport
      .execute('net_version');
  }
}
