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
import { isHexNumber } from '../../../../test/types';

describe('ethapi.web3', () => {
  const ethapi = createHttpApi();

  describe('clientVersion', () => {
    it('returns the client version', () => {
      return ethapi.web3.clientVersion().then((version) => {
        const [client] = version.split('/');

        expect(client === 'Parity' || client === 'Geth').to.be.ok;
      });
    });
  });

  describe('sha3', () => {
    it('returns a keccak256 sha', () => {
      const sha = '0xa7916fac4f538170f7cd12c148552e2cba9fcd72329a2dd5b07a6fa906488ddf';
      const hexStr = 'baz()'.split('').map((char) => char.charCodeAt(0).toString(16)).join('');

      return ethapi.web3.sha3(`0x${hexStr}`).then((hash) => {
        expect(isHexNumber(hash)).to.be.true;
        expect(hash).to.equal(sha);
      });
    });
  });
});
