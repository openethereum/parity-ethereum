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
import { connect } from 'react-redux';
import { bindActionCreators } from 'redux';

import { fetchTokens } from '~/redux/providers/tokensActions';
import styles from './methodDecoding.css';

class TokenValue extends Component {
  static propTypes = {
    id: PropTypes.string.isRequired,
    value: PropTypes.object.isRequired,

    fetchTokens: PropTypes.func,
    token: PropTypes.object
  };

  componentWillMount () {
    const { token } = this.props;

    if (!token.fetched) {
      this.props.fetchTokens([ token.index ]);
    }
  }

  render () {
    const { token, value } = this.props;

    if (!token.format) {
      console.warn('token with no format', token);
    }

    const format = token.format
      ? token.format
      : 1;

    const precision = token.format
      ? 5
      : 0;

    const tag = token.format
      ? token.tag
      : 'TOKENS';

    return (
      <span className={ styles.tokenValue }>
        { value.div(format).toFormat(precision) }<small> { tag }</small>
      </span>
    );
  }
}

function mapStateToProps (initState, initProps) {
  const { id } = initProps;
  let token = Object.assign({}, initState.tokens[id]);

  if (token.fetched) {
    return () => ({ token });
  }

  let update = true;

  return (state) => {
    if (update) {
      const { tokens } = state;
      const nextToken = tokens[id];

      if (nextToken.fetched) {
        token = Object.assign({}, nextToken);
        update = false;
      }
    }

    return { token };
  };
}

function mapDispatchToProps (dispatch) {
  return bindActionCreators({
    fetchTokens
  }, dispatch);
}

export default connect(
  mapStateToProps,
  mapDispatchToProps
)(TokenValue);
