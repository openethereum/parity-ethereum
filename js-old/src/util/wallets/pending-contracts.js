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

import store from 'store';

const LS_PENDING_CONTRACTS_KEY = '_parity::wallets::pendingContracts';

export default class PendingContracts {
  static getPendingContracts () {
    return store.get(LS_PENDING_CONTRACTS_KEY) || {};
  }

  static setPendingContracts (contracts = {}) {
    return store.set(LS_PENDING_CONTRACTS_KEY, contracts);
  }

  static removePendingContract (operationHash) {
    const nextContracts = PendingContracts.getPendingContracts();

    delete nextContracts[operationHash];
    PendingContracts.setPendingContracts(nextContracts);
  }

  static addPendingContract (address, operationHash, metadata) {
    const nextContracts = {
      ...PendingContracts.getPendingContracts(),
      [ operationHash ]: {
        address,
        metadata,
        operationHash
      }
    };

    PendingContracts.setPendingContracts(nextContracts);
  }
}
