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

import { createHttpApi } from '../../../../test/e2e/ethapi';

describe('ethapi.ethcore', () => {
  const ethapi = createHttpApi();

  describe('gasFloorTarget', () => {
    it('returns and translates the target', () => {
      return ethapi.ethcore.gasFloorTarget().then((value) => {
        expect(value.gt(0)).to.be.true;
      });
    });
  });

  describe('netChain', () => {
    it('returns and the chain', () => {
      return ethapi.ethcore.netChain().then((value) => {
        expect(value).to.equal('morden');
      });
    });
  });

  describe('netPort', () => {
    it('returns and translates the port', () => {
      return ethapi.ethcore.netPort().then((value) => {
        expect(value.gt(0)).to.be.true;
      });
    });
  });

  describe('transactionsLimit', () => {
    it('returns and translates the limit', () => {
      return ethapi.ethcore.transactionsLimit().then((value) => {
        expect(value.gt(0)).to.be.true;
      });
    });
  });

  describe('rpcSettings', () => {
    it('returns and translates the settings', () => {
      return ethapi.ethcore.rpcSettings().then((value) => {
        expect(value).to.be.ok;
      });
    });
  });
});
