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

export default {
  button: {
    delete: `delete`,
    edit: `edit`,
    export: `export`,
    faucet: `Kovan ETH`,
    forget: `forget`,
    password: `password`,
    shapeshift: `shapeshift`,
    transfer: `transfer`,
    verify: `verify`
  },
  export: {
    info: `Export your account as a JSON file. Please enter the password linked with this account.`,
    password: {
      hint: `The password specified when creating this account`,
      label: `Account password`
    },
    setPassword: {
      hint: `Enter password Here`,
      label: `Password`
    },
    title: `Export Account`
  },
  external: {
    confirmDelete: `Are you sure you want to remove the following external address from your account list?`
  },
  hardware: {
    confirmDelete: `Are you sure you want to remove the following hardware address from your account list?`
  },
  header: {
    outgoingTransactions: `{count} outgoing transactions`,
    uuid: `uuid: {uuid}`
  },
  title: `Account Management`,
  transactions: {
    poweredBy: `Transaction list powered by {etherscan}`,
    title: `transactions`
  }
};
