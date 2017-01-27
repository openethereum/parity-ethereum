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

describe('ethapi.parity', () => {
  const ethapi = createHttpApi();

  describe('chainStatus', () => {
    it('returns and translates the status', () => {
      return ethapi.parity.chainStatus().then((value) => {
        expect(value).to.be.ok;
      });
    });
  });

  describe('gasFloorTarget', () => {
    it('returns and translates the target', () => {
      return ethapi.parity.gasFloorTarget().then((value) => {
        expect(value.gt(0)).to.be.true;
      });
    });
  });

  describe('gasPriceHistogram', () => {
    it('returns and translates the target', () => {
      return ethapi.parity.gasPriceHistogram().then((result) => {
        expect(Object.keys(result)).to.deep.equal(['bucketBounds', 'counts']);
        expect(result.bucketBounds.length > 0).to.be.true;
        expect(result.counts.length > 0).to.be.true;
      });
    });
  });

  describe('netChain', () => {
    it('returns and the chain', () => {
      return ethapi.parity.netChain().then((value) => {
        expect(value).to.equal('morden');
      });
    });
  });

  describe('netPort', () => {
    it('returns and translates the port', () => {
      return ethapi.parity.netPort().then((value) => {
        expect(value.gt(0)).to.be.true;
      });
    });
  });

  describe('transactionsLimit', () => {
    it('returns and translates the limit', () => {
      return ethapi.parity.transactionsLimit().then((value) => {
        expect(value.gt(0)).to.be.true;
      });
    });
  });

  describe('rpcSettings', () => {
    it('returns and translates the settings', () => {
      return ethapi.parity.rpcSettings().then((value) => {
        expect(value).to.be.ok;
      });
    });
  });
});
