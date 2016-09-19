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

import BigNumber from 'bignumber.js';
import React, { Component, PropTypes } from 'react';

import styles from './balance.css';

export default class Balance extends Component {
  static contextTypes = {
    api: PropTypes.object
  }

  static propTypes = {
    balance: PropTypes.object
  }

  render () {
    const { api } = this.context;
    const { balance } = this.props;

    if (!balance) {
      return null;
    }

    let body = balance.tokens
      .filter((balance) => new BigNumber(balance.value).gt(0))
      .map((balance) => {
        const token = balance.token;
        const value = token.format
          ? new BigNumber(balance.value).div(new BigNumber(token.format)).toFormat(3)
          : api.format.fromWei(balance.value).toFormat(3);

        return (
          <div
            className={ styles.balance }
            key={ token.tag }>
            <img
              src={ token.image }
              alt={ token.name } />
            <div>{ value }<small> { token.tag }</small></div>
          </div>
        );
      });

    if (!body.length) {
      body = (
        <div className={ styles.empty }>
          There are no balances associated with this account
        </div>
      );
    }

    return (
      <div className={ styles.balances }>
        { body }
      </div>
    );
  }
}
