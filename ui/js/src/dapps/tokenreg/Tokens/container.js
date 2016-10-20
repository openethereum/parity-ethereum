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
import { connect } from 'react-redux';

import Tokens from './tokens';

import { loadTokens, queryTokenMeta, unregisterToken, addTokenMeta } from './actions';

class TokensContainer extends Component {
  static propTypes = {
    isOwner: PropTypes.bool,
    isLoading: PropTypes.bool,
    tokens: PropTypes.array,
    tokenCount: PropTypes.number,
    onLoadTokens: PropTypes.func,
    accounts: PropTypes.array
  };

  componentDidMount () {
    this.props.onLoadTokens();
  }

  render () {
    console.log(this.props);
    return (
      <Tokens
        { ...this.props }
      />
    );
  }
}

const mapStateToProps = (state) => {
  const { list } = state.accounts;
  const { isLoading, tokens, tokenCount } = state.tokens;

  const { isOwner } = state.status.contract;

  return { isLoading, tokens, tokenCount, isOwner, accounts: list };
};

const mapDispatchToProps = (dispatch) => {
  return {
    onLoadTokens: () => {
      dispatch(loadTokens());
    },

    handleMetaLookup: (index, query) => {
      dispatch(queryTokenMeta(index, query));
    },

    handleUnregister: (index) => {
      dispatch(unregisterToken(index));
    },

    handleAddMeta: (index, key, value) => {
      dispatch(addTokenMeta(index, key, value));
    }
  };
};

export default connect(
  mapStateToProps,
  mapDispatchToProps
)(TokensContainer);
