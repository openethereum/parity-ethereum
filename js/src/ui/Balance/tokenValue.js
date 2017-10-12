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
import { connect } from 'react-redux';
import { bindActionCreators } from 'redux';

import { fetchTokens } from '~/redux/providers/tokensActions';
import TokenImage from '~/ui/TokenImage';

import styles from './balance.css';

class TokenValue extends Component {
  static propTypes = {
    token: PropTypes.object.isRequired,
    value: PropTypes.object.isRequired,

    // Redux injection
    fetchTokens: PropTypes.func.isRequired,

    showOnlyEth: PropTypes.bool
  };

  componentWillMount () {
    const { token } = this.props;

    if (token.native) {
      return;
    }

    if (!token.fetched) {
      if (!Number.isFinite(token.index)) {
        return console.warn('no token index', token);
      }

      this.props.fetchTokens([ token.index ]);
    }
  }

  render () {
    const { token, showOnlyEth, value } = this.props;

    const isEthToken = token.native;
    const isFullToken = !showOnlyEth || isEthToken;

    const bnf = new BigNumber(token.format || 1);
    let decimals = 0;

    if (bnf.gte(1000)) {
      decimals = 3;
    } else if (bnf.gte(100)) {
      decimals = 2;
    } else if (bnf.gte(10)) {
      decimals = 1;
    }

    const rawValue = new BigNumber(value).div(bnf);
    const classNames = [styles.balance];

    if (isFullToken) {
      classNames.push(styles.full);
    }

    return (
      <div className={ classNames.join(' ') }>
        <TokenImage token={ token } />
        {
          isFullToken
          ? [
            <div className={ styles.value } key='value'>
              <span title={ `${rawValue.toFormat()} ${token.tag}` }>
                { rawValue.toFormat(decimals) }
              </span>
            </div>,
            <div className={ styles.tag } key='tag'>
              { token.tag }
            </div>
          ]
          : null
        }
      </div>
    );
  }
}

function mapDispatchToProps (dispatch) {
  return bindActionCreators({
    fetchTokens
  }, dispatch);
}

export default connect(
  null,
  mapDispatchToProps
)(TokenValue);
