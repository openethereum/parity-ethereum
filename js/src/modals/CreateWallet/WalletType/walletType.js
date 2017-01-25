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

import React, { Component, PropTypes } from 'react';

import { RadioButtons } from '~/ui';
import { walletSourceURL } from '~/contracts/code/wallet';

// import styles from '../createWallet.css';

export default class WalletType extends Component {
  static propTypes = {
    onChange: PropTypes.func.isRequired,
    type: PropTypes.string.isRequired
  };

  render () {
    const { type } = this.props;

    return (
      <RadioButtons
        name='contractType'
        value={ type }
        values={ this.getTypes() }
        onChange={ this.onTypeChange }
      />
    );
  }

  getTypes () {
    return [
      {
        label: 'Multi-Sig wallet', key: 'MULTISIG',
        description: (
          <span>
            <span>Create/Deploy a </span>
            <a href={ walletSourceURL } target='_blank'>
              standard multi-signature
            </a>
            <span> Wallet</span>
          </span>
        )
      },
      {
        label: 'Watch a wallet', key: 'WATCH',
        description: 'Add an existing wallet to your accounts'
      }
    ];
  }

  onTypeChange = (type) => {
    this.props.onChange(type.key);
  }
}
