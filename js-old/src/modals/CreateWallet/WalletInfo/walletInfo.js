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
import { FormattedMessage } from 'react-intl';

import { fromWei } from '@parity/api/lib/util/wei';
import { CompletedStep, IdentityIcon, CopyToClipboard } from '~/ui';

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
          <span>
            <FormattedMessage
              id='createWallet.info.created'
              defaultMessage='{name} has been {deployedOrAdded} at '
              values={ {
                name: <code>{ name }</code>,
                deployedOrAdded: deployed
                  ? (
                    <FormattedMessage
                      id='createWallet.info.deployed'
                      defaultMessage='deployed'
                    />
                  )
                  : (
                    <FormattedMessage
                      id='createWallet.info.added'
                      defaultMessage='added'
                    />
                  )
              } }
            />
          </span>
        </div>
        <div>
          <CopyToClipboard
            data={ address }
            label={
              <FormattedMessage
                id='createWallet.info.copyAddress'
                defaultMessage='copy address to clipboard'
              />
            }
          />
          <IdentityIcon
            address={ address }
            className={ styles.identityicon }
            center
            inline
          />
          <div className={ styles.address }>{ address }</div>
        </div>
        <div>
          <FormattedMessage
            id='createWallet.info.owners'
            defaultMessage='The following are wallet owners'
          />
        </div>
        <div>
          { this.renderOwners() }
        </div>
        <p>
          <FormattedMessage
            id='createWallet.info.numOwners'
            defaultMessage='{numOwners} owners are required to confirm a transaction.'
            values={ {
              numOwners: <code>{ required }</code>
            } }
          />
        </p>
        <p>
          <FormattedMessage
            id='createWallet.info.dayLimit'
            defaultMessage='The daily limit is set to {dayLimit} ETH.'
            values={ {
              dayLimit: <code>{ fromWei(daylimit).toFormat() }</code>
            } }
          />
        </p>
      </CompletedStep>
    );
  }

  renderOwners () {
    const { account, owners, deployed } = this.props;

    return []
      .concat(deployed ? account : null, owners)
      .filter((account) => account)
      .map((address, id) => {
        return (
          <div
            className={ styles.owner }
            key={ id }
          >
            <IdentityIcon
              address={ address }
              className={ styles.identityicon }
              center
              inline
            />
            <div className={ styles.address }>
              { this.addressToString(address) }
            </div>
          </div>
        );
      });
  }

  addressToString (address) {
    const { accounts } = this.props;

    if (accounts[address]) {
      return accounts[address].name || address;
    }

    return address;
  }
}
