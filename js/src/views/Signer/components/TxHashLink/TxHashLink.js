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

import { getTxLink } from '../util/transaction';

export default class TxHashLink extends Component {

  static propTypes = {
    txHash: PropTypes.string.isRequired,
    chain: PropTypes.string.isRequired,
    children: PropTypes.node,
    className: PropTypes.string
  }

  state = {
    link: null
  };

  componentWillMount () {
    const { txHash, chain } = this.props;
    this.updateLink(txHash, chain);
  }

  componentWillReceiveProps (nextProps) {
    const { txHash, chain } = nextProps;
    this.updateLink(txHash, chain);
  }

  render () {
    const { children, txHash, className } = this.props;
    const { link } = this.state;

    return (
      <a
        href={ link }
        target='_blank'
        className={ className }>
        { children || txHash }
      </a>
    );
  }

  updateLink (txHash, chain) {
    const link = getTxLink(txHash, chain);
    this.setState({ link });
  }

}
