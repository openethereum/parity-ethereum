// Copyright 2015, 2016 Ethcore (UK) Ltd.
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

import { addressLink } from '../../../../../3rdparty/etherscan/links';
import styles from './AccountLink.css';

export default class AccountLink extends Component {
  static propTypes = {
    chain: PropTypes.string.isRequired,
    address: PropTypes.string.isRequired,
    className: PropTypes.string,
    children: PropTypes.node
  }

  state = {
    link: null
  };

  componentWillMount () {
    const { address, chain } = this.props;

    this.updateLink(address, chain);
  }

  componentWillReceiveProps (nextProps) {
    const { address, chain } = nextProps;

    this.updateLink(address, chain);
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

  updateLink (address, chain) {
    const link = addressLink(address, chain === 'morden' || chain === 'testnet');

    this.setState({
      link
    });
  }
}
