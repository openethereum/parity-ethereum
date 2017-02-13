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
import { MenuItem } from 'material-ui';
import { isEqual } from 'lodash';

import { Select } from '~/ui/Form';
import TokenImage from '~/ui/TokenImage';

import styles from '../transfer.css';

export default class TokenSelect extends Component {
  static contextTypes = {
    api: PropTypes.object
  };

  static propTypes = {
    onChange: PropTypes.func.isRequired,
    balance: PropTypes.object.isRequired,
    tag: PropTypes.string.isRequired
  };

  componentWillMount () {
    this.computeTokens();
  }

  componentWillReceiveProps (nextProps) {
    const prevTokens = this.props.balance.tokens.map((t) => `${t.token.tag}_${t.value.toNumber()}`);
    const nextTokens = nextProps.balance.tokens.map((t) => `${t.token.tag}_${t.value.toNumber()}`);

    if (!isEqual(prevTokens, nextTokens)) {
      this.computeTokens(nextProps);
    }
  }

  computeTokens (props = this.props) {
    const { api } = this.context;
    const { balance } = this.props;

    const items = balance.tokens
      .filter((token, index) => !index || token.value.gt(0))
      .map((balance, index) => {
        const token = balance.token;
        const isEth = index === 0;

        let value = 0;

        if (isEth) {
          value = api.util.fromWei(balance.value).toFormat(3);
        } else {
          const format = balance.token.format || 1;
          const decimals = format === 1 ? 0 : Math.min(3, Math.floor(format / 10));

          value = new BigNumber(balance.value).div(format).toFormat(decimals);
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

        return (
          <MenuItem
            key={ `${index}_${token.tag}` }
            value={ token.tag }
            label={ label }
          >
            { label }
          </MenuItem>
        );
      });

    this.setState({ items });
  }

  render () {
    const { tag, onChange } = this.props;
    const { items } = this.state;

    return (
      <Select
        className={ styles.tokenSelect }
        label='type of token transfer'
        hint='type of token to transfer'
        value={ tag }
        onChange={ onChange }
      >
        { items }
      </Select>
    );
  }
}
