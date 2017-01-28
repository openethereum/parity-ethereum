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

import { addressLink } from '~/3rdparty/etherscan/links';
import styles from './accountLink.css';

export default class AccountLink extends Component {
  static propTypes = {
    isTest: PropTypes.bool.isRequired,
    address: PropTypes.string.isRequired,
    className: PropTypes.string,
    children: PropTypes.node
  }

  state = {
    link: null
  };

  componentWillMount () {
    const { address, isTest } = this.props;

    this.updateLink(address, isTest);
  }

  componentWillReceiveProps (nextProps) {
    const { address, isTest } = nextProps;

    this.updateLink(address, isTest);
  }

  render () {
    const { children, address, className } = this.props;

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

  updateLink (address, isTest) {
    const link = addressLink(address, isTest);

    this.setState({
      link
    });
  }
}
