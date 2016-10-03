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
    managerInstance: PropTypes.object.isRequired
  }

  static propTypes = {
    address: PropTypes.string.isRequired
  }

  state = {
    tokens: []
  }

  componentDidMount () {
    this.loadTokens();
  }

  render () {
    const { address } = this.props;
    const { tokens } = this.state;

    if (!tokens.length) {
      return null;
    }

    return (
      <div className={ styles.info }>
        <div className={ styles.owner }>{ address }</div>
        { this.renderTokens() }
      </div>
    );
  }

  renderTokens () {
    const { tokens } = this.state;

    return tokens.map((token) => (
      <Token
        key={ token.address }
        address={ token.address }
        tokenreg={ token.tokenreg } />
    ));
  }

  loadTokens () {
    const { managerInstance } = this.context;
    const { address } = this.props;

    managerInstance
      .countByOwner.call({}, [address])
      .then((count) => {
        const promises = [];

        for (let index = 0; count.gt(index); index++) {
          promises.push(managerInstance.getByOwner.call({}, [address, index]));
        }

        return Promise.all(promises);
      })
      .then((tokens) => {
        this.setState({
          tokens: tokens.map(([address, _owner, tokenreg]) => {
            return { address, tokenreg };
          })
        });
      });
  }
}
