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
import { Link } from 'react-router';

import styles from './accountLink.css';

class AccountLink extends Component {
  static propTypes = {
    accountAddresses: PropTypes.array.isRequired,
    address: PropTypes.string.isRequired,
    className: PropTypes.string,
    children: PropTypes.node,
    externalLink: PropTypes.string.isRequired
  }

  state = {
    link: null
  };

  componentWillMount () {
    const { address, externalLink } = this.props;

    this.updateLink(address, externalLink);
  }

  componentWillReceiveProps (nextProps) {
    const { address, externalLink } = nextProps;

    this.updateLink(address, externalLink);
  }

  render () {
    const { children, address, className, externalLink } = this.props;

    if (externalLink) {
      return (
        <a
          href={ this.state.link }
          target='_blank'
          className={ `${styles.container} ${className}` }
        >
          { children || address }
        </a>
      );
    }

    return (
      <Link
        className={ `${styles.container} ${className}` }
        to={ this.state.link }
      >
        { children || address }
      </Link>
    );
  }

  updateLink (address, externalLink) {
    const { accountAddresses } = this.props;
    const isAccount = accountAddresses.includes(address);

    let link = isAccount
      ? `/accounts/${address}`
      : `/addresses/${address}`;

    if (externalLink) {
      const path = externalLink.replace(/\/+$/, '');

      link = `${path}/#${link}`;
    }

    this.setState({
      link
    });
  }
}

function mapStateToProps (initState) {
  const { accounts } = initState.personal;
  const accountAddresses = Object.keys(accounts);

  return () => {
    return { accountAddresses };
  };
}

export default connect(
  mapStateToProps,
  null
)(AccountLink);
