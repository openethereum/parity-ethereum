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

import { registry as registryAbi, registry2 as registryAbi2 } from '~/contracts/abi';

import { api } from './parity.js';
import * as addresses from './addresses/actions.js';
import * as accounts from './Accounts/actions.js';
import * as lookup from './Lookup/actions.js';
import * as events from './Events/actions.js';
import * as names from './Names/actions.js';
import * as records from './Records/actions.js';
import * as reverse from './Reverse/actions.js';

export { addresses, accounts, lookup, events, names, records, reverse };

const REGISTRY_V1_HASHES = [
  '0x34f7c51bbb1b1902fbdabfdf04811100f5c9f998f26dd535d2f6f977492c748e', // ropsten
  '0x64c3ee34851517a9faecd995c102b339f03e564ad6772dc43a26f993238b20ec' // homestead
];

export const setNetVersion = (netVersion) => ({ type: 'set netVersion', netVersion });

export const fetchIsTestnet = () => (dispatch) =>
  api.net.version()
    .then((netVersion) => {
      dispatch(setNetVersion(netVersion));
    })
    .catch((err) => {
      console.error('could not check if testnet');
      if (err) {
        console.error(err.stack);
      }
    });

export const setContract = (contract) => ({ type: 'set contract', contract });

export const fetchContract = () => (dispatch) =>
  api.parity
    .registryAddress()
    .then((address) => {
      return api.eth
        .getCode(address)
        .then((code) => {
          const codeHash = api.util.sha3(code);
          const isVersion1 = REGISTRY_V1_HASHES.includes(codeHash);

          console.log(`registry at ${address}, code ${codeHash}, version ${isVersion1 ? 1 : 2}`);

          const contract = api.newContract(
            isVersion1
              ? registryAbi
              : registryAbi2,
            address
          );

          dispatch(setContract(contract));
          dispatch(fetchFee());
          dispatch(fetchOwner());
        });
    })
    .catch((err) => {
      console.error('could not fetch contract');
      if (err) {
        console.error(err.stack);
      }
    });

export const setFee = (fee) => ({ type: 'set fee', fee });

const fetchFee = () => (dispatch, getState) => {
  const { contract } = getState();

  if (!contract) {
    return;
  }

  contract.instance.fee.call()
    .then((fee) => dispatch(setFee(fee)))
    .catch((err) => {
      console.error('could not fetch fee');
      if (err) {
        console.error(err.stack);
      }
    });
};

export const setOwner = (owner) => ({ type: 'set owner', owner });

export const fetchOwner = () => (dispatch, getState) => {
  const { contract } = getState();

  if (!contract) {
    return;
  }

  contract.instance.owner.call()
    .then((owner) => dispatch(setOwner(owner)))
    .catch((err) => {
      console.error('could not fetch owner');
      if (err) {
        console.error(err.stack);
      }
    });
};
