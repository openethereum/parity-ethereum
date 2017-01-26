// Copyright 2015-2017 Parity Technologies (UK) Ltd.
// This file is part of Parity.

// Parity is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Parity is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Parity.  If not, see <http://www.gnu.org/licenses/>.

import { inBlockNumber, inData, inHex, inNumber16, inOptions, inTraceFilter, inTraceType } from '../../format/input';
import { outTraces, outTraceReplay } from '../../format/output';

export default class Trace {
  constructor (transport) {
    this._transport = transport;
  }

  block (blockNumber = 'latest') {
    return this._transport
      .execute('trace_block', inBlockNumber(blockNumber))
      .then(outTraces);
  }

  call (options, blockNumber = 'latest', whatTrace = ['trace']) {
    return this._transport
      .execute('trace_call', inOptions(options), inBlockNumber(blockNumber), inTraceType(whatTrace))
      .then(outTraceReplay);
  }

  filter (filterObj) {
    return this._transport
      .execute('trace_filter', inTraceFilter(filterObj))
      .then(outTraces);
  }

  get (txHash, position) {
    return this._transport
      .execute('trace_get', inHex(txHash), inNumber16(position))
      .then(outTraces);
  }

  rawTransaction (data, whatTrace = ['trace']) {
    return this._transport
      .execute('trace_rawTransaction', inData(data), inTraceType(whatTrace))
      .then(outTraceReplay);
  }

  replayTransaction (txHash, whatTrace = ['trace']) {
    return this._transport
      .execute('trace_replayTransaction', txHash, inTraceType(whatTrace))
      .then(outTraceReplay);
  }

  transaction (txHash) {
    return this._transport
      .execute('trace_transaction', inHex(txHash))
      .then(outTraces);
  }
}
