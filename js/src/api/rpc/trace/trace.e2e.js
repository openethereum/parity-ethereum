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

import { createHttpApi } from '../../../../test/e2e/ethapi';

describe('ethapi.trace', () => {
  const ethapi = createHttpApi();

  describe('block', () => {
    it('returns the latest block traces', () => {
      return ethapi.trace.block().then((traces) => {
        expect(traces).to.be.ok;
      });
    });

    it('returns traces for a specified block', () => {
      return ethapi.trace.block('0x65432').then((traces) => {
        expect(traces).to.be.ok;
      });
    });
  });

  describe('replayTransaction', () => {
    it('returns traces for a specific transaction', () => {
      return ethapi.eth.getBlockByNumber().then((latestBlock) => {
        return ethapi.trace.replayTransaction(latestBlock.transactions[0]).then((traces) => {
          expect(traces).to.be.ok;
        });
      });
    });
  });
});
