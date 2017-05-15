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

import ApplicationStore from '../Application/application.store';
import LookupStore from '../Lookup/lookup.store';
import Hash from './hash';
import etherscanUrl from '../util/etherscan-url';
import IdentityIcon from '../IdentityIcon';

import styles from './address.css';

export default class Address extends Component {
  static propTypes = {
    address: PropTypes.string.isRequired,
    big: PropTypes.bool,
    className: PropTypes.string,
    key: PropTypes.string,
    shortenHash: PropTypes.bool
  };

  static defaultProps = {
    big: false,
    key: 'address',
    shortenHash: true
  };

  applicationStore = ApplicationStore.get();
  lookupStore = LookupStore.get();

  state = {
    account: null
  };

  componentWillMount () {
    this.setAccount();
  }

  componentWillReceiveProps (nextProps) {
    if (this.props.address !== nextProps.address) {
      this.setAccount(nextProps);
    }
  }

  render () {
    const { address, big, className, key } = this.props;
    const classes = [ styles.container, className ];

    if (big) {
      classes.push(styles.big);
    }

    return (
      <div
        key={ key }
        className={ classes.join(' ') }
      >
        <IdentityIcon
          address={ address }
          big={ big }
          className={ [ styles.icon, styles.align ].join(' ') }
          onClick={ this.handleClick }
        />
        { this.renderCaption() }
      </div>
    );
  }

  renderCaption () {
    const { address, shortenHash } = this.props;

    if (this.state.account) {
      const { name } = this.state.account;

      return (
        <div>
          <abbr
            title={ address }
            className={ [ styles.address, styles.align ].join(' ') }
          >
            { name || address }
          </abbr>
        </div>
      );
    }

    return (
      <code className={ [ styles.address, styles.align ].join(' ') }>
        { shortenHash ? (
          <Hash
            hash={ address }
          />
        ) : address }
      </code>
    );
  }

  setAccount (props = this.props) {
    const { address } = props;
    const lcAddress = address.toLowerCase();
    const account = this.applicationStore.accounts.find((a) => a.address.toLowerCase() === lcAddress);

    this.setState({ account });
  }

  handleClick = () => {
    this.lookupStore.updateInput(this.props.address);
  };
}
