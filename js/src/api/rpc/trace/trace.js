// Copyright 2015, 2016 Ethcore (UK) Ltd.
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

import { inBlockNumber, inHex, inNumber16, inTraceFilter } from '../../format/input';
import { outTrace } from '../../format/output';

export default class Trace {
  constructor (transport) {
    this._transport = transport;
  }

  filter (filterObj) {
    return this._transport
      .execute('trace_filter', inTraceFilter(filterObj))
      .then(traces => traces.map(trace => outTrace(trace)));
  }

  get (txHash, position) {
    return this._transport
      .execute('trace_get', inHex(txHash), inNumber16(position))
      .then(trace => outTrace(trace));
  }

  transaction (txHash) {
    return this._transport
      .execute('trace_transaction', inHex(txHash))
      .then(traces => traces.map(trace => outTrace(trace)));
  }

  block (blockNumber = 'latest') {
    return this._transport
      .execute('trace_block', inBlockNumber(blockNumber))
      .then(traces => traces.map(trace => outTrace(trace)));
  }
}
