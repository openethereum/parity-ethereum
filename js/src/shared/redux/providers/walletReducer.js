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

import { handleActions } from 'redux-actions';

const initialState = {
  wallets: {},
  walletsAddresses: [],
  filterSubId: null,
  contract: null
};

export default handleActions({
  updateWallets: (state, action) => {
    const { wallets, walletsAddresses, filterSubId } = action;

    return {
      ...state,
      wallets, walletsAddresses, filterSubId
    };
  },

  updateWalletsDetails: (state, action) => {
    const { wallets } = action;
    const prevWallets = state.wallets;

    const nextWallets = { ...prevWallets };

    Object.values(wallets).forEach((wallet) => {
      const prevWallet = prevWallets[wallet.address] || {};

      nextWallets[wallet.address] = {
        instance: (state.contract && state.contract.instance) || null,
        ...prevWallet,
        ...wallet
      };
    });

    return {
      ...state,
      wallets: nextWallets
    };
  },

  setWalletContract: (state, action) => {
    const { contract } = action;

    return {
      ...state,
      contract
    };
  },

  setOperationPendingState: (state, action) => {
    const { address, operation, isPending } = action;
    const { wallets } = state;

    const wallet = { ...wallets[address] };

    wallet.confirmations = wallet.confirmations.map((conf) => {
      if (conf.operation === operation) {
        conf.pending = isPending;
      }

      return conf;
    });

    return {
      ...state,
      wallets: {
        ...wallets,
        [ address ]: wallet
      }
    };
  }
}, initialState);
