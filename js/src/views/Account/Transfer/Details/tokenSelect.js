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
import React, { Component } from 'react';
import PropTypes from 'prop-types';
import { connect } from 'react-redux';
import { isEqual } from 'lodash';

import { Dropdown, TokenImage } from '@parity/ui';

import styles from '../transfer.css';

class TokenSelect extends Component {
  static contextTypes = {
    api: PropTypes.object
  };

  static propTypes = {
    onChange: PropTypes.func.isRequired,
    balance: PropTypes.object.isRequired,
    tokens: PropTypes.object.isRequired,
    value: PropTypes.string.isRequired
  };

  componentWillMount () {
    this.computeTokens();
  }

  componentWillReceiveProps (nextProps) {
    const prevTokens = Object.keys(this.props.balance)
      .map((tokenId) => `${tokenId}_${this.props.balance[tokenId].toNumber()}`);
    const nextTokens = Object.keys(nextProps.balance)
      .map((tokenId) => `${tokenId}_${nextProps.balance[tokenId].toNumber()}`);

    if (!isEqual(prevTokens, nextTokens)) {
      this.computeTokens(nextProps);
    }
  }

  computeTokens (props = this.props) {
    const { api } = this.context;
    const { balance, tokens } = this.props;

    const items = Object
      .keys(balance)
      .map((tokenId) => {
        const token = tokens[tokenId];
        const tokenValue = balance[tokenId];
        const isEth = token.native;

        if (!isEth && tokenValue.eq(0)) {
          return null;
        }

        let value = 0;

        if (isEth) {
          value = api.util.fromWei(tokenValue).toFormat(3);
        } else {
          const format = token.format || 1;
          const decimals = format === 1 ? 0 : Math.min(3, Math.floor(format / 10));

          value = new BigNumber(tokenValue).div(format).toFormat(decimals);
        }

        const label = (
          <div className={ styles.token }>
            <TokenImage token={ token } />
            <div className={ styles.tokenname }>
              { token.name }
            </div>
            <div className={ styles.tokenbalance }>
              { value }<small> { token.tag }</small>
            </div>
          </div>
        );

        return {
          key: tokenId,
          text: token.name,
          value: token.id,
          content: label
        };
      })
      .filter((node) => node);

    this.setState({ items });
  }

  render () {
    const { onChange, value } = this.props;
    const { items } = this.state;

    return (
      <Dropdown
        className={ styles.tokenSelect }
        label='type of token transfer'
        hint='type of token to transfer'
        value={ value }
        onChange={ onChange }
        options={ items }
      />
    );
  }
}

function mapStateToProps (state) {
  const { tokens } = state;

  return { tokens };
}

export default connect(
  mapStateToProps,
  null
)(TokenSelect);
