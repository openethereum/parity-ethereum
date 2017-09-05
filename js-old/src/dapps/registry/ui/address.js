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
import { connect } from 'react-redux';

import Hash from './hash';
import etherscanUrl from '../util/etherscan-url';
import IdentityIcon from '../IdentityIcon';
import { nullableProptype } from '~/util/proptypes';

import styles from './address.css';

class Address extends Component {
  static propTypes = {
    address: PropTypes.string.isRequired,
    account: nullableProptype(PropTypes.object.isRequired),
    netVersion: PropTypes.string.isRequired,
    key: PropTypes.string,
    shortenHash: PropTypes.bool
  };

  static defaultProps = {
    key: 'address',
    shortenHash: true
  };

  render () {
    const { address, key } = this.props;

    return (
      <div
        key={ key }
        className={ styles.container }
      >
        <IdentityIcon
          address={ address }
          className={ styles.align }
        />
        { this.renderCaption() }
      </div>
    );
  }

  renderCaption () {
    const { address, account, netVersion, shortenHash } = this.props;

    if (account) {
      const { name } = account;

      return (
        <a
          className={ styles.link }
          href={ etherscanUrl(address, false, netVersion) }
          target='_blank'
        >
          <abbr
            title={ address }
            className={ styles.align }
          >
            { name || address }
          </abbr>
        </a>
      );
    }

    return (
      <code className={ styles.align }>
        { shortenHash ? (
          <Hash
            hash={ address }
            linked
          />
        ) : address }
      </code>
    );
  }
}

function mapStateToProps (initState, initProps) {
  const { accounts, contacts } = initState;

  const allAccounts = Object.assign({}, accounts.all, contacts);

  // Add lower case addresses to map
  Object
    .keys(allAccounts)
    .forEach((address) => {
      allAccounts[address.toLowerCase()] = allAccounts[address];
    });

  return (state, props) => {
    const { netVersion } = state;
    const { address = '' } = props;

    const account = allAccounts[address] || null;

    return {
      account,
      netVersion
    };
  };
}

export default connect(
  mapStateToProps
)(Address);
