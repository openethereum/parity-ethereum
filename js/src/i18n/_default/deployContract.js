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
    cancel: `Cancel`,
    close: `Close`,
    create: `Create`,
    next: `Next`
  },
  details: {
    abi: {
      hint: `the abi of the contract to deploy or solc combined-output`,
      label: `abi / solc combined-output`
    },
    address: {
      hint: `the owner account for this contract`,
      label: `from account (contract owner)`
    },
    advanced: {
      label: `advanced sending options`
    },
    amount: {
      hint: `the amount to transfer to the contract`,
      label: `amount to transfer (in {tag})`
    },
    code: {
      hint: `the compiled code of the contract to deploy`,
      label: `code`
    },
    contract: {
      label: `select a contract`
    },
    description: {
      hint: `a description for the contract`,
      label: `contract description (optional)`
    },
    name: {
      hint: `a name for the deployed contract`,
      label: `contract name`
    }
  },
  owner: {
    noneSelected: `a valid account as the contract owner needs to be selected`
  },
  parameters: {
    choose: `Choose the contract parameters`
  },
  title: {
    details: `contract details`,
    extras: `extra information`,
    parameters: `contract parameters`
  }
};
