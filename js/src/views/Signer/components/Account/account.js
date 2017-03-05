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

import { IdentityIcon, IdentityName } from '~/ui';
import AccountLink from './AccountLink';

import styles from './account.css';

export default class Account extends Component {
  static propTypes = {
    address: PropTypes.string.isRequired,
    className: PropTypes.string,
    disabled: PropTypes.bool,
    externalLink: PropTypes.string.isRequired,
    netVersion: PropTypes.string.isRequired,
    balance: PropTypes.object // eth BigNumber, not required since it mght take time to fetch
  };

  state = {
    balanceDisplay: '?'
  };

  componentWillMount () {
    this.updateBalanceDisplay(this.props.balance);
  }

  componentWillReceiveProps (nextProps) {
    if (nextProps.balance === this.props.balance) {
      return;
    }
    this.updateBalanceDisplay(nextProps.balance);
  }

  updateBalanceDisplay (balance) {
    this.setState({
      balanceDisplay: balance ? balance.div(1e18).toFormat(3) : '?'
    });
  }

  render () {
    const { address, className, disabled, externalLink, netVersion } = this.props;

    return (
      <div className={ `${styles.acc} ${className}` }>
        <AccountLink
          address={ address }
          externalLink={ externalLink }
          netVersion={ netVersion }
        >
          <IdentityIcon
            center
            disabled={ disabled }
            address={ address }
          />
        </AccountLink>
        { this.renderName() }
        { this.renderBalance() }
      </div>
    );
  }

  renderBalance () {
    const { balanceDisplay } = this.state;

    return (
      <span> <strong>{ balanceDisplay }</strong> <small>ETH</small></span>
    );
  }

  renderName () {
    const { address, externalLink, netVersion } = this.props;
    const name = <IdentityName address={ address } empty />;

    if (!name) {
      return (
        <AccountLink
          address={ address }
          externalLink={ externalLink }
          netVersion={ netVersion }
        >
          [{ this.shortAddress(address) }]
        </AccountLink>
      );
    }

    return (
      <AccountLink
        address={ address }
        externalLink={ externalLink }
        netVersion={ netVersion }
      >
        <span>
          <span className={ styles.name }>{ name }</span>
          <span className={ styles.address }>[{ this.tinyAddress(address) }]</span>
        </span>
      </AccountLink>
    );
  }

  tinyAddress () {
    const { address } = this.props;
    const len = address.length;

    return address.slice(2, 4) + '..' + address.slice(len - 2);
  }

  shortAddress () {
    const { address } = this.props;
    const len = address.length;

    return address.slice(2, 8) + '..' + address.slice(len - 7);
  }
}
