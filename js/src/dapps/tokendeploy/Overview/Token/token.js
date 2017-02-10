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

import { totalSupply, getCoin } from '../../services';
import styles from './token.css';

export default class Token extends Component {
  static propTypes = {
    address: PropTypes.string.isRequired,
    tokenreg: PropTypes.string.isRequired
  }

  state = {
    coin: null,
    totalSupply: null
  }

  componentDidMount () {
    this.lookupToken();
  }

  render () {
    const { coin, totalSupply } = this.state;

    if (!coin) {
      return null;
    }

    return (
      <div className={ styles.info }>
        <div className={ styles.tla }>{ coin.tla }</div>
        <div className={ styles.name }>{ coin.name }</div>
        <div className={ styles.supply }>
          <div>{ totalSupply.div(1000000).toFormat(0) }</div>
          <div className={ styles.info }>total supply</div>
        </div>
      </div>
    );
  }

  lookupToken () {
    const { address, tokenreg } = this.props;

    Promise
      .all([
        getCoin(tokenreg, address),
        totalSupply(address)
      ])
      .then(([coin, totalSupply]) => {
        this.setState({ coin, totalSupply });
      });
  }
}
