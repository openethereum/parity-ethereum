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

import { isEqual, intersection } from 'lodash';

import BalancesProvider from './balances';
import { updateTokensFilter } from './balancesActions';
import { attachWallets } from './walletActions';

import Contract from '~/api/contract';
import MethodDecodingStore from '~/ui/MethodDecoding/methodDecodingStore';
import WalletsUtils from '~/util/wallets';
import { foundationWallet as WalletAbi } from '~/contracts/abi';

export function personalAccountsInfo (accountsInfo) {
  const accounts = {};
  const contacts = {};
  const contracts = {};
  const hardware = {};
  const wallets = {};

  Object.keys(accountsInfo || {})
    .map((address) => Object.assign({}, accountsInfo[address], { address }))
    .filter((account) => account.meta && (account.uuid || !account.meta.deleted))
    .forEach((account) => {
      if (account.uuid) {
        accounts[account.address] = account;
      } else if (account.meta.wallet) {
        account.wallet = true;
        wallets[account.address] = account;
      } else if (account.meta.contract) {
        account.contract = true;
        contracts[account.address] = account;
      } else if (account.meta.hardware) {
        account.hardware = true;
        hardware[account.address] = account;
        accounts[account.address] = account;
      } else if (account.meta.external) {
        account.external = true;
        accounts[account.address] = account;
      } else {
        contacts[account.address] = account;
      }
    });

  // Load user contracts for Method Decoding
  MethodDecodingStore.loadContracts(contracts);

  return (dispatch, getState) => {
    const { api } = getState();

    const _fetchOwners = Object
      .values(wallets)
      .map((wallet) => {
        const walletContract = new Contract(api, WalletAbi);

        return WalletsUtils.fetchOwners(walletContract.at(wallet.address));
      });

    Promise
      .all(_fetchOwners)
      .then((walletsOwners) => {
        return Object
          .values(wallets)
          .map((wallet, index) => {
            wallet.owners = walletsOwners[index].map((owner) => ({
              address: owner,
              name: accountsInfo[owner] && accountsInfo[owner].name || owner
            }));

            return wallet;
          });
      })
      .catch(() => {
        return [];
      })
      .then((_wallets) => {
        // We want to separate owned wallets and other wallets
        // However, wallets can be owned by wallets, that can
        // be owned by an account...
        let otherWallets = [].concat(_wallets);
        let prevLength;
        let nextLength;

        // If no more other wallets, or if the size decreased, continue...
        do {
          prevLength = otherWallets.length;

          otherWallets = otherWallets
            .map((wallet) => {
              const addresses = Object.keys(accounts);
              const owners = wallet.owners.map((o) => o.address);

              // Owners ∩ Addresses not null : Wallet is owned
              // by one of the accounts
              if (intersection(owners, addresses).length > 0) {
                accounts[wallet.address] = wallet;
                return false;
              }

              return wallet;
            })
            .filter((wallet) => wallet);

          nextLength = otherWallets.length;
        } while (nextLength < prevLength);

        // And other wallets to contacts...
        otherWallets.forEach((wallet) => {
          contacts[wallet.address] = wallet;
        });

        // Cache the _real_ accounts for
        // WalletsUtils (used for sending transactions)
        WalletsUtils.cacheAccounts(accounts);

        dispatch(_personalAccountsInfo({
          accountsInfo,
          accounts,
          contacts,
          contracts,
          hardware
        }));
        dispatch(attachWallets(wallets));

        BalancesProvider.get().fetchAllBalances({
          force: true
        });
      })
      .catch((error) => {
        console.warn('personalAccountsInfo', error);
        throw error;
      });
  };
}

function _personalAccountsInfo (data) {
  return {
    type: 'personalAccountsInfo',
    ...data
  };
}

export function _setVisibleAccounts (addresses) {
  return {
    type: 'setVisibleAccounts',
    addresses
  };
}

export function setVisibleAccounts (addresses) {
  return (dispatch, getState) => {
    const { visibleAccounts } = getState().personal;

    if (isEqual(addresses.sort(), visibleAccounts.sort())) {
      return;
    }

    dispatch(_setVisibleAccounts(addresses));

    // Don't update the balances if no new addresses displayed
    if (addresses.length === 0) {
      return;
    }

    // Update the Tokens filter to take into account the new
    // addresses
    dispatch(updateTokensFilter());

    BalancesProvider.get().fetchBalances({
      force: true
    });
  };
}
