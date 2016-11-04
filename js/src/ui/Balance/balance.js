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
import { connect } from 'react-redux';
import { bindActionCreators } from 'redux';

import unknownImage from '../../../assets/images/contracts/unknown-64x64.png';
import styles from './balance.css';

class Balance extends Component {
  static contextTypes = {
    api: PropTypes.object
  };

  static propTypes = {
    images: PropTypes.object.isRequired,
    balances: PropTypes.array
  };

  static defaultProps = {
    balances: []
  };

  render () {
    const tokens = this.getTokens();

    if (tokens.length === 0) {
      return (
        <div className={ styles.empty }>
          There are no balances associated with this account
        </div>
      );
    }

    let body = tokens
      .map((token) => this.renderToken(token));

    return (
      <div className={ styles.balances }>
        { body }
      </div>
    );
  }

  renderToken (token) {
    const { name, tag, value, image } = token;

    return (
      <div
        className={ styles.balance }
        key={ tag }>
        <img
          src={ image }
          alt={ name } />
        <div className={ styles.balanceValue }>
          <span title={ value }> { value } </span>
        </div>
        <div className={ styles.balanceTag }> { tag } </div>
      </div>
    );
  }

  getTokens () {
    const { api } = this.context;
    const { images, balances } = this.props;

    return balances
      .filter((balance) => new BigNumber(balance.value).gt(0))
      .map((balance) => {
        const token = balance.token;

        const value = token.format
          ? new BigNumber(balance.value).div(new BigNumber(token.format)).toFormat(3)
          : api.util.fromWei(balance.value).toFormat(3);

        const image = token.image || images[token.address] || unknownImage;

        return {
          name: token.name,
          tag: token.tag,
          value,
          image
        };
      });
  }

}

function mapStateToProps (_, initProps) {
  return (state) => {
    const { images } = state;
    return { images };
  };
}

function mapDispatchToProps (dispatch) {
  return bindActionCreators({}, dispatch);
}

export default connect(
  mapStateToProps,
  mapDispatchToProps
)(Balance);
