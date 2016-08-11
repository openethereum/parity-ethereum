import { inBlockNumber, inHex, inNumber16 } from '../../format/input';

export default class Trace {
  constructor (transport) {
    this._transport = transport;
  }

  filter (filterObj) {
    return this._transport
      .execute('trace_filter', filterObj);
  }

  get (txHash, position) {
    return this._transport
      .execute('trace_get', inHex(txHash), inNumber16(position));
  }

  transaction (txHash) {
    return this._transport
      .execute('trace_transaction', inHex(txHash));
  }

  block (blockNumber = 'latest') {
    return this._transport
      .execute('trace_block', inBlockNumber(blockNumber));
  }
}
