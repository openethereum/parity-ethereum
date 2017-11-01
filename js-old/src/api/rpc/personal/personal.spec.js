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

import { TEST_HTTP_URL, mockHttp } from '../../../../test/mockRpc';

import Http from '../../transport/http';
import Personal from './personal';

const instance = new Personal(new Http(TEST_HTTP_URL, -1));

describe('rpc/Personal', () => {
  const account = '0x63cf90d3f0410092fc0fca41846f596223979195';
  const checksum = '0x63Cf90D3f0410092FC0fca41846f596223979195';
  let scope;

  describe('listAccounts', () => {
    it('retrieves a list of available accounts', () => {
      scope = mockHttp([{ method: 'personal_listAccounts', reply: { result: [account] } }]);

      return instance.listAccounts().then((result) => {
        expect(result).to.deep.equal([checksum]);
      });
    });

    it('returns an empty list when none available', () => {
      scope = mockHttp([{ method: 'personal_listAccounts', reply: { result: null } }]);

      return instance.listAccounts().then((result) => {
        expect(result).to.deep.equal([]);
      });
    });
  });

  describe('newAccount', () => {
    it('passes the password, returning the address', () => {
      scope = mockHttp([{ method: 'personal_newAccount', reply: { result: account } }]);

      return instance.newAccount('password').then((result) => {
        expect(scope.body.personal_newAccount.params).to.deep.equal(['password']);
        expect(result).to.equal(checksum);
      });
    });
  });

  describe('unlockAccount', () => {
    beforeEach(() => {
      scope = mockHttp([{ method: 'personal_unlockAccount', reply: { result: [] } }]);
    });

    it('passes account, password & duration', () => {
      return instance.unlockAccount(account, 'password', 0xf).then(() => {
        expect(scope.body.personal_unlockAccount.params).to.deep.equal([account, 'password', 15]);
      });
    });

    it('provides a default duration when not specified', () => {
      return instance.unlockAccount(account, 'password').then(() => {
        expect(scope.body.personal_unlockAccount.params).to.deep.equal([account, 'password', 1]);
      });
    });
  });
});
