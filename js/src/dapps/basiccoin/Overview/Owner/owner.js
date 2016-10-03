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

import Token from '../Token';
import styles from './owner.css';

export default class Owner extends Component {
  static contextTypes = {
    accounts: PropTypes.object.isRequired,
    managerInstance: PropTypes.object.isRequired
  }

  static propTypes = {
    address: PropTypes.string.isRequired,
    tokens: PropTypes.array.isRequired
  }

  state = {
    tokens: []
  }

  render () {
    const { accounts } = this.context;
    const { address, tokens } = this.props;

    if (!tokens.length) {
      return null;
    }

    return (
      <div className={ styles.info }>
        <div className={ styles.owner }>
          { accounts[address].name }
        </div>
        { this.renderTokens() }
      </div>
    );
  }

  renderTokens () {
    const { tokens } = this.props;

    return tokens.map((token) => (
      <Token
        key={ token.address }
        address={ token.address }
        tokenreg={ token.tokenreg } />
    ));
  }
}
