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

import BigNumber from 'bignumber.js';
import React, { Component, PropTypes } from 'react';
import { FormattedMessage } from 'react-intl';

import TokenImage from '~/ui/TokenImage';

import styles from './balance.css';

export default class Balance extends Component {
  static contextTypes = {
    api: PropTypes.object
  };

  static propTypes = {
    balance: PropTypes.object,
    className: PropTypes.string,
    showOnlyEth: PropTypes.bool,
    showZeroValues: PropTypes.bool
  };

  static defaultProps = {
    showOnlyEth: false,
    showZeroValues: false
  };

  render () {
    const { api } = this.context;
    const { balance, className, showOnlyEth } = this.props;

    if (!balance || !balance.tokens) {
      return null;
    }

    let body = balance.tokens
      .filter((balance) => {
        const isEthToken = (balance.token.tag || '').toLowerCase() === 'eth';
        const hasBalance = new BigNumber(balance.value).gt(0);

        return hasBalance || isEthToken;
      })
      .map((balance, index) => {
        const isFullToken = !showOnlyEth || (balance.token.tag || '').toLowerCase() === 'eth';
        const token = balance.token;

        let value;

        if (token.format) {
          const bnf = new BigNumber(token.format);

          let decimals = 0;

          if (bnf.gte(1000)) {
            decimals = 3;
          } else if (bnf.gte(100)) {
            decimals = 2;
          } else if (bnf.gte(10)) {
            decimals = 1;
          }

          value = new BigNumber(balance.value).div(bnf).toFormat(decimals);
        } else {
          value = api.util.fromWei(balance.value).toFormat(3);
        }

        const classNames = [styles.balance];
        let details = null;

        if (isFullToken) {
          classNames.push(styles.full);
          details = [
            <div
              className={ styles.value }
              key='value'
            >
              <span title={ value }>
                { value }
              </span>
            </div>,
            <div
              className={ styles.tag }
              key='tag'
            >
              { token.tag }
            </div>
          ];
        }

        return (
          <div
            className={ classNames.join(' ') }
            key={ `${index}_${token.tag}` }
          >
            <TokenImage token={ token } />
            { details }
          </div>
        );
      });

    if (!body.length) {
      body = (
        <div className={ styles.empty }>
          <FormattedMessage
            id='ui.balance.none'
            defaultMessage='No balances associated with this account'
          />
        </div>
      );
    }

    return (
      <div
        className={
          [
            styles.balances,
            showOnlyEth
              ? ''
              : styles.full,
            className
          ].join(' ')
        }
      >
        { body }
      </div>
    );
  }
}
