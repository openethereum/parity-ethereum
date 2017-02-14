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
  abi: {
    hint: `de abi van het contract`,
    label: `contract abi`
  },
  abiType: {
    custom: {
      description: `Contract aangemaakt met een custom ABI`,
      label: `Custom Contract`
    },
    multisigWallet: {
      description: `Ethereum Multisig contract {link}`,
      label: `Multisig Wallet`,
      link: `zie contract code`
    },
    token: {
      description: `Een standaard {erc20} token`,
      erc20: `ERC 20`,
      label: `Token`
    }
  },
  address: {
    hint: `het netwerk adres van het contract`,
    label: `netwerk adres`
  },
  button: {
    add: `Voeg Contract toe`,
    cancel: `Annuleer`,
    next: `Volgende`,
    prev: `Terug`
  },
  description: {
    hint: `een uitgebreide omschrijving van het contract`,
    label: `(optioneel) contract beschrijving`
  },
  name: {
    hint: `een beschrijvende naam van het contract`,
    label: `contract naam`
  },
  title: {
    details: `voer contract details in`,
    type: `kies een contract type`
  }
};
