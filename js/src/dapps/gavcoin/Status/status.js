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

import { formatBlockNumber, formatCoins, formatEth } from '../format';

import styles from './status.css';

export default class Status extends Component {
  static propTypes = {
    address: PropTypes.string,
    gavBalance: PropTypes.object,
    blockNumber: PropTypes.object,
    totalSupply: PropTypes.object,
    remaining: PropTypes.object,
    price: PropTypes.object,
    children: PropTypes.node
  }

  render () {
    const { blockNumber, gavBalance, totalSupply, remaining, price } = this.props;

    if (!totalSupply) {
      return null;
    }

    return (
      <div className={ styles.status }>
        <div className={ styles.item }>
          <div className={ styles.heading }>&nbsp;</div>
          <div className={ styles.hero }>
            { formatCoins(remaining, -1) }
          </div>
          <div className={ styles.byline }>
            available for { formatEth(price) }ETH
          </div>
        </div>
        <div className={ styles.item }>
          <div className={ styles.heading }>GAVcoin</div>
          <div className={ styles.hero }>
            { formatCoins(totalSupply, -1) }
          </div>
          <div className={ styles.byline }>
            total at { formatBlockNumber(blockNumber) }
          </div>
        </div>
        <div className={ styles.item }>
          <div className={ styles.heading }>&nbsp;</div>
          <div className={ styles.hero }>
            { formatCoins(gavBalance, -1) }
          </div>
          <div className={ styles.byline }>
            coin balance
          </div>
        </div>
        { this.props.children }
      </div>
    );
  }
}
