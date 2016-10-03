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

import styles from './token.css';

export default class Token extends Component {
  static contextTypes = {
    registryInstance: PropTypes.object.isRequired,
    tokenregInstance: PropTypes.object.isRequired
  }

  static propTypes = {
    address: PropTypes.string.isRequired,
    tokenreg: PropTypes.string.isRequired
  }

  state = {
    id: null,
    tla: null,
    base: null,
    name: null,
    owner: null,
    isGlobal: false
  }

  componentDidMount () {
    this.lookupToken();
  }

  render () {
    const { address } = this.props;
    const { tla, name, isGlobal, base } = this.state;

    if (!base) {
      return null;
    }

    return (
      <div className={ styles.info }>
        <div className={ styles.address }>{ address }</div>
        <div className={ styles.tla }>{ tla }</div>
        <div className={ styles.name }>{ name }</div>
        <div className={ styles.global }>{ isGlobal ? 'global' : 'local' }</div>
      </div>
    );
  }

  lookupToken () {
    const { registryInstance, tokenregInstance } = this.context;
    const { address, tokenreg } = this.props;
    const isGlobal = tokenreg === tokenregInstance.address;
    const registry = isGlobal ? tokenregInstance : registryInstance;

    registry.fromAddress
      .call({}, [address])
      .then(([id, tla, base, name, owner]) => {
        this.setState({ id, tla, base, name, owner, isGlobal });
      });
  }
}
