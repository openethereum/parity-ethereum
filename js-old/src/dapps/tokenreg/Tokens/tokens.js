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

import Token from './Token';
import Loading from '../Loading';

import styles from './tokens.css';

export default class Tokens extends Component {
  static propTypes = {
    isLoading: PropTypes.bool.isRequired,
    tokens: PropTypes.array
  };

  render () {
    const { isLoading, tokens } = this.props;
    const loading = isLoading ? (<Loading size={ 2 } />) : null;

    return (
      <div className={ styles.tokens }>
        { this.renderTokens(tokens) }
        { loading }
      </div>
    );
  }

  renderTokens (tokens) {
    return tokens.map((token) => {
      return (
        <Token
          key={ token.tla }
          tla={ token.tla }
        />
      );
    });
  }
}
