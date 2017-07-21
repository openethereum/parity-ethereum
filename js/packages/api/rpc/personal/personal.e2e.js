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
import { isAddress, isBoolean } from '../../../../test/types';

describe.skip('ethapi.personal', () => {
  const ethapi = createHttpApi();
  const password = 'P@55word';
  let address;

  describe('newAccount', () => {
    it('creates a new account', () => {
      return ethapi.personal.newAccount(password).then((_address) => {
        address = _address;
        expect(isAddress(address)).to.be.ok;
      });
    });
  });

  describe('listAccounts', () => {
    it('has the newly-created account', () => {
      return ethapi.personal.listAccounts(password).then((accounts) => {
        expect(accounts.filter((_address) => _address === address)).to.deep.equal([address]);
        accounts.forEach((account) => {
          expect(isAddress(account)).to.be.true;
        });
      });
    });
  });

  describe('unlockAccount', () => {
    it('unlocks the newly-created account', () => {
      return ethapi.personal.unlockAccount(address, password).then((result) => {
        expect(isBoolean(result)).to.be.true;
        expect(result).to.be.true;
      });
    });
  });
});
