// Copyright 2015, 2016 Parity Technologies (UK) Ltd.
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

import { isEqual } from 'lodash';

import { fetchBalances } from './balancesActions';
import { attachWallets } from './walletActions';

import MethodDecodingStore from '~/ui/MethodDecoding/methodDecodingStore';

export function personalAccountsInfo (accountsInfo) {
  const accounts = {};
  const contacts = {};
  const contracts = {};
  const wallets = {};

  Object.keys(accountsInfo || {})
    .map((address) => Object.assign({}, accountsInfo[address], { address }))
    .filter((account) => account.uuid || !account.meta.deleted)
    .forEach((account) => {
      if (account.uuid) {
        accounts[account.address] = account;
      } else if (account.meta.wallet) {
        account.wallet = true;
        wallets[account.address] = account;
      } else if (account.meta.contract) {
        contracts[account.address] = account;
      } else {
        contacts[account.address] = account;
      }
    });

  // Load user contracts for Method Decoding
  MethodDecodingStore.loadContracts(contracts);

  return (dispatch) => {
    const data = {
      accountsInfo,
      accounts, contacts, contracts, wallets
    };

    dispatch(_personalAccountsInfo(data));
    dispatch(attachWallets(wallets));
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

    dispatch(fetchBalances(addresses));
    dispatch(_setVisibleAccounts(addresses));
  };
}
