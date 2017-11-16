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

/* eslint-disable no-unused-expressions */

const JsonRpcBase = require('@parity/api/lib/transport/jsonRpcBase');

const LocalAccountsMiddleware = require('./localAccountsMiddleware');

const RPC_RESPONSE = Symbol('RPC response');
const ADDRESS = '0x00a329c0648769a73afac7f9381e08fb43dbea72';
const SECRET = '0x4d5db4107d237df6a3d58ee5f70ae63d73d7658d4026f2eefd2f204c81682cb7';
const PASSWORD = 'password';

const FOO_PHRASE = 'foobar';
const FOO_PASSWORD = 'foopass';
const FOO_ADDRESS = '0x007ef7ac1058e5955e366ab9d6b6c4ebcc937e7e';

class MockedTransport extends JsonRpcBase {
  _execute (method, params) {
    return RPC_RESPONSE;
  }
}

// Skip till all CI runs on Node 8+
describe.skip('api/local/LocalAccountsMiddleware', function () {
  this.timeout(30000);

  let transport;

  beforeEach(() => {
    transport = new MockedTransport();
    transport.addMiddleware(LocalAccountsMiddleware);

    // Same as `parity_newAccountFromPhrase` with empty phrase
    return transport
      .execute('parity_newAccountFromSecret', [SECRET, PASSWORD])
      .catch((_err) => {
        // Ignore the error - all instances of LocalAccountsMiddleware
        // share account storage
      });
  });

  it('registers all necessary methods', () => {
    return Promise
      .all([
        'eth_accounts',
        'eth_coinbase',
        'parity_accountsInfo',
        'parity_allAccountsInfo',
        'parity_changePassword',
        'parity_checkRequest',
        'parity_defaultAccount',
        'parity_generateSecretPhrase',
        'parity_getNewDappsAddresses',
        'parity_hardwareAccountsInfo',
        'parity_newAccountFromPhrase',
        'parity_newAccountFromSecret',
        'parity_setAccountMeta',
        'parity_setAccountName',
        'parity_postTransaction',
        'parity_phraseToAddress',
        'parity_useLocalAccounts',
        'parity_listGethAccounts',
        'parity_listOpenedVaults',
        'parity_listRecentDapps',
        'parity_listVaults',
        'parity_killAccount',
        'parity_testPassword',
        'signer_confirmRequest',
        'signer_rejectRequest',
        'signer_requestsToConfirm'
      ].map((method) => {
        return transport
          .execute(method)
          .then((result) => {
            expect(result).not.to.be.equal(RPC_RESPONSE);
          })
          // Some errors are expected here since we are calling methods
          // without parameters.
          .catch((_) => {});
      }));
  });

  it('allows non-registered methods through', () => {
    return transport
      .execute('eth_getBalance', ['0x407d73d8a49eeb85d32cf465507dd71d507100c1'])
      .then((result) => {
        expect(result).to.be.equal(RPC_RESPONSE);
      });
  });

  it('can handle `eth_accounts`', () => {
    return transport
      .execute('eth_accounts')
      .then((accounts) => {
        expect(accounts.length).to.be.equal(1);
        expect(accounts[0]).to.be.equal(ADDRESS);
      });
  });

  it('can handle `parity_defaultAccount`', () => {
    return transport
      .execute('parity_defaultAccount')
      .then((address) => {
        expect(address).to.be.equal(ADDRESS);
      });
  });

  it('can handle `parity_phraseToAddress`', () => {
    return transport
      .execute('parity_phraseToAddress', [''])
      .then((address) => {
        expect(address).to.be.equal(ADDRESS);

        return transport.execute('parity_phraseToAddress', [FOO_PHRASE]);
      })
      .then((address) => {
        expect(address).to.be.equal(FOO_ADDRESS);
      });
  });

  it('can create and kill an account', () => {
    return transport
      .execute('parity_newAccountFromPhrase', [FOO_PHRASE, FOO_PASSWORD])
      .then((address) => {
        expect(address).to.be.equal(FOO_ADDRESS);

        return transport.execute('eth_accounts');
      })
      .then((accounts) => {
        expect(accounts.length).to.be.equal(2);
        expect(accounts.includes(FOO_ADDRESS)).to.be.true;

        return transport.execute('parity_killAccount', [FOO_ADDRESS, FOO_PASSWORD]);
      })
      .then((result) => {
        expect(result).to.be.true;

        return transport.execute('eth_accounts');
      })
      .then((accounts) => {
        expect(accounts.length).to.be.equal(1);
        expect(accounts.includes(FOO_ADDRESS)).to.be.false;
      });
  });
});
