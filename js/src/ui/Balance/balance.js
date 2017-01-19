// Copyright 2015, 2016 Parity Technologies (UK) Ltd.
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
import { connect } from 'react-redux';

import unknownImage from '../../../assets/images/contracts/unknown-64x64.png';
import styles from './balance.css';

class Balance extends Component {
  static contextTypes = {
    api: PropTypes.object
  }

  static propTypes = {
    balance: PropTypes.object,
    images: PropTypes.object.isRequired
  }

  render () {
    const { api } = this.context;
    const { balance, images } = this.props;

    if (!balance) {
      return null;
    }

    let body = (balance.tokens || [])
      .filter((balance) => new BigNumber(balance.value).gt(0))
      .map((balance) => {
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

        const imageurl = token.image || images[token.address];
        let imagesrc = unknownImage;

        if (imageurl) {
          const host = /^(\/)?api/.test(imageurl)
            ? api.dappsUrl
            : '';

          imagesrc = `${host}${imageurl}`;
        }

        return (
          <div
            className={ styles.balance }
            key={ token.tag }
          >
            <img
              src={ imagesrc }
              alt={ token.name }
            />
            <div className={ styles.balanceValue }>
              <span title={ value }> { value } </span>
            </div>
            <div className={ styles.balanceTag }> { token.tag } </div>
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

function mapStateToProps (state) {
  const { images } = state;

  return { images };
}

export default connect(
  mapStateToProps,
  null
)(Balance);
