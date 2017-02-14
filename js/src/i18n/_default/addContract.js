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
  title: {
    type: `choose a contract type`,
    details: `enter contract details`
  },
  button: {
    cancel: `Cancel`,
    next: `Next`,
    prev: `Back`,
    add: `Add Contract`
  },
  address: {
    hint: `the network address for the contract`,
    label: `network address`
  },
  name: {
    hint: `a descriptive name for the contract`,
    label: `contract name`
  },
  description: {
    hint: `an expanded description for the entry`,
    label: `(optional) contract description`
  },
  abi: {
    hint: `the abi for the contract`,
    label: `contract abi`
  },
  abiType: {
    token: {
      description: `A standard {erc20} token`,
      erc20: `ERC 20`,
      label: `Token`
    },
    multisigWallet: {
      description: `Ethereum Multisig contract {link}`,
      link: `see contract code`,
      label: `Multisig Wallet`
    },
    custom: {
      description: `Contract created from custom ABI`,
      label: `Custom Contract`
    }
  }
};
