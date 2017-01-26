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

import { CompletedStep, IdentityIcon, CopyToClipboard } from '~/ui';
import { fromWei } from '~/api/util/wei';

import styles from '../createWallet.css';

export default class WalletInfo extends Component {
  static propTypes = {
    accounts: PropTypes.object.isRequired,
    account: PropTypes.string.isRequired,
    name: PropTypes.string.isRequired,
    address: PropTypes.string.isRequired,
    owners: PropTypes.array.isRequired,
    required: PropTypes.oneOfType([
      PropTypes.string,
      PropTypes.object,
      PropTypes.number
    ]).isRequired,
    daylimit: PropTypes.oneOfType([
      PropTypes.string,
      PropTypes.object,
      PropTypes.number
    ]).isRequired,

    deployed: PropTypes.bool
  };

  render () {
    const { address, required, daylimit, name, deployed } = this.props;

    return (
      <CompletedStep>
        <div>
          <code>{ name }</code>
          <span> has been </span>
          <span> { deployed ? 'deployed' : 'added' } at </span>
        </div>
        <div>
          <CopyToClipboard data={ address } label='copy address to clipboard' />
          <IdentityIcon address={ address } inline center className={ styles.identityicon } />
          <div className={ styles.address }>{ address }</div>
        </div>
        <div>with the following owners</div>
        <div>
          { this.renderOwners() }
        </div>
        <p>
          <code>{ required }</code> owners are required to confirm a transaction.
        </p>
        <p>
          The daily limit is set to <code>{ fromWei(daylimit).toFormat() }</code> ETH.
        </p>
      </CompletedStep>
    );
  }

  renderOwners () {
    const { account, owners, deployed } = this.props;

    return [].concat(deployed ? account : null, owners).filter((a) => a).map((address, id) => (
      <div key={ id } className={ styles.owner }>
        <IdentityIcon address={ address } inline center className={ styles.identityicon } />
        <div className={ styles.address }>{ this.addressToString(address) }</div>
      </div>
    ));
  }

  addressToString (address) {
    const { accounts } = this.props;

    if (accounts[address]) {
      return accounts[address].name || address;
    }

    return address;
  }
}
