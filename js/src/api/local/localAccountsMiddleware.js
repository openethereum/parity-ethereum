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

import EthereumTx from 'ethereumjs-tx';
import accounts from './accounts';
import transactions from './transactions';
import { Middleware } from '../transport';
import { inNumber16 } from '../format/input';
import { phraseToWallet, phraseToAddress, verifySecret } from './ethkey';
import { randomPhrase } from '@parity/wordlist';

export default class LocalAccountsMiddleware extends Middleware {
  constructor (transport) {
    super(transport);

    const register = this.register.bind(this);

    register('eth_accounts', () => {
      return accounts.mapArray((account) => account.address);
    });

    register('eth_coinbase', () => {
      return accounts.lastAddress;
    });

    register('parity_accountsInfo', () => {
      return accounts.mapObject(({ name }) => {
        return { name };
      });
    });

    register('parity_allAccountsInfo', () => {
      return accounts.mapObject(({ name, meta, uuid }) => {
        return { name, meta, uuid };
      });
    });

    register('parity_changePassword', ([address, oldPassword, newPassword]) => {
      const account = accounts.get(address);

      return account
        .decryptPrivateKey(oldPassword)
        .then((privateKey) => {
          if (!privateKey) {
            return false;
          }

          account.changePassword(privateKey, newPassword);

          return true;
        });
    });

    register('parity_checkRequest', ([id]) => {
      return transactions.hash(id) || Promise.resolve(null);
    });

    register('parity_defaultAccount', () => {
      return accounts.lastAddress;
    });

    register('parity_generateSecretPhrase', () => {
      return randomPhrase(12);
    });

    register('parity_getNewDappsAddresses', () => {
      return [];
    });

    register('parity_hardwareAccountsInfo', () => {
      return {};
    });

    register('parity_newAccountFromPhrase', ([phrase, password]) => {
      return phraseToWallet(phrase)
        .then((wallet) => {
          return accounts.create(wallet.secret, password);
        });
    });

    register('parity_newAccountFromSecret', ([secret, password]) => {
      return verifySecret(secret)
        .then((isValid) => {
          if (!isValid) {
            throw new Error('Invalid secret key');
          }

          return accounts.create(secret, password);
        });
    });

    register('parity_setAccountMeta', ([address, meta]) => {
      accounts.get(address).meta = meta;

      return true;
    });

    register('parity_setAccountName', ([address, name]) => {
      accounts.get(address).name = name;

      return true;
    });

    register('parity_postTransaction', ([tx]) => {
      if (!tx.from) {
        tx.from = accounts.lastAddress;
      }

      tx.nonce = null;
      tx.condition = null;

      return transactions.add(tx);
    });

    register('parity_phraseToAddress', ([phrase]) => {
      return phraseToAddress(phrase);
    });

    register('parity_useLocalAccounts', () => {
      return true;
    });

    register('parity_listGethAccounts', () => {
      return [];
    });

    register('parity_listRecentDapps', () => {
      return {};
    });

    register('parity_killAccount', ([address, password]) => {
      return accounts.remove(address, password);
    });

    register('parity_testPassword', ([address, password]) => {
      const account = accounts.get(address);

      return account.isValidPassword(password);
    });

    register('signer_confirmRequest', ([id, modify, password]) => {
      const {
        gasPrice,
        gas: gasLimit,
        from,
        to,
        value,
        data
      } = Object.assign(transactions.get(id), modify);

      transactions.lock(id);

      const account = accounts.get(from);

      return Promise.all([
        this.rpcRequest('parity_nextNonce', [from]),
        account.decryptPrivateKey(password)
      ])
      .then(([nonce, privateKey]) => {
        const tx = new EthereumTx({
          nonce,
          to,
          data,
          gasLimit: inNumber16(gasLimit),
          gasPrice: inNumber16(gasPrice),
          value: inNumber16(value)
        });

        tx.sign(privateKey);

        const serializedTx = `0x${tx.serialize().toString('hex')}`;

        return this.rpcRequest('eth_sendRawTransaction', [serializedTx]);
      })
      .then((hash) => {
        transactions.confirm(id, hash);

        return {};
      });
    });

    register('signer_rejectRequest', ([id]) => {
      return transactions.reject(id);
    });

    register('signer_requestsToConfirm', () => {
      return transactions.requestsToConfirm();
    });
  }
}
