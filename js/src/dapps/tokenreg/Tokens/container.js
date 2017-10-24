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

import Tokens from './tokens';

import { loadTokens } from './actions';

class TokensContainer extends Component {
  static propTypes = {
    isLoading: PropTypes.bool,
    tokens: PropTypes.array,
    onLoadTokens: PropTypes.func
  };

  componentDidMount () {
    this.props.onLoadTokens();
  }

  render () {
    return (
      <Tokens
        { ...this.props }
      />
    );
  }
}

const mapStateToProps = (state) => {
  const { isLoading, tokens } = state.tokens;

  const filteredTokens = tokens
    .filter((token) => token && token.tla)
    .map((token) => ({ tla: token.tla, owner: token.owner }));

  return { isLoading, tokens: filteredTokens };
};

const mapDispatchToProps = (dispatch) => {
  return {
    onLoadTokens: () => {
      dispatch(loadTokens());
    }
  };
};

export default connect(
  mapStateToProps,
  mapDispatchToProps
)(TokensContainer);
