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

import React from 'react';
import { FormattedMessage } from 'react-intl';

import { eip20, foundationWallet } from '~/contracts/abi';

const ABI_TYPES = [
  {
    description: (
      <FormattedMessage
        id='addContract.abiType.token.description'
        defaultMessage='A standard {erc20} token'
        values={ {
          erc20: (
            <a href='https://github.com/ethereum/EIPs/issues/20' target='_blank'>
              <FormattedMessage
                id='addContract.abiType.token.erc20'
                defaultMessage='ERC 20'
              />
            </a>
          )
        } }
      />
    ),
    label: (
      <FormattedMessage
        id='addContract.abiType.token.label'
        defaultMessage='Token'
      />
    ),
    readOnly: true,
    type: 'token',
    value: JSON.stringify(eip20)
  },
  {
    description: (
      <FormattedMessage
        id='addContract.abiType.multisigWallet.description'
        defaultMessage='Ethereum Multisig contract {link}'
        values={ {
          link: (
            <a href='https://github.com/ethereum/dapp-bin/blob/master/wallet/wallet.sol' target='_blank'>
              <FormattedMessage
                id='addContract.abiType.multisigWallet.link'
                defaultMessage='see contract code'
              />
            </a>
          )
        } }
      />
    ),
    label: (
      <FormattedMessage
        id='addContract.abiType.multisigWallet.label'
        defaultMessage='Multisig Wallet'
      />
    ),
    readOnly: true,
    type: 'multisig',
    value: JSON.stringify(foundationWallet)
  },
  {
    description: (
      <FormattedMessage
        id='addContract.abiType.custom.description'
        defaultMessage='Contract created from custom ABI'
      />
    ),
    label: (
      <FormattedMessage
        id='addContract.abiType.custom.label'
        defaultMessage='Custom Contract'
      />
    ),
    type: 'custom',
    value: ''
  }
];

export {
  ABI_TYPES
};
