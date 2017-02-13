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

import IdentityIcon from '../../IdentityIcon';
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
      <tr className={ styles.info }>
        <td className={ styles.owner }>
          <div>
            <span>{ accounts[address].name }</span>
            <IdentityIcon
              className={ styles.icon }
              address={ address }
            />
          </div>
        </td>
        <td className={ styles.tokens }>
          { this.renderTokens() }
        </td>
      </tr>
    );
  }

  renderTokens () {
    const { tokens } = this.props;

    return tokens.map((token) => (
      <div key={ token.address }>
        <Token
          address={ token.address }
          tokenreg={ token.tokenreg }
        />
        <div className={ styles.byline }>
          { token.address }
        </div>
      </div>
    ));
  }
}
