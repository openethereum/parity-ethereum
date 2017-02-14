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

import { loadOwnedTokens } from '../services';
import Container from '../Container';
import Owner from './Owner';

import styles from './overview.css';

export default class Overview extends Component {
  static contextTypes = {
    accounts: PropTypes.object.isRequired,
    managerInstance: PropTypes.object.isRequired
  }

  state = {
    loading: true,
    total: new BigNumber(0),
    tokenOwners: []
  }

  componentDidMount () {
    this.loadOwners();
  }

  render () {
    const { loading } = this.state;

    return (
      <Container>
        { loading ? this.renderLoading() : this.renderBody() }
      </Container>
    );
  }

  renderLoading () {
    return (
      <div className={ styles.statusHeader }>
        Loading tokens
      </div>
    );
  }

  renderBody () {
    const { total } = this.state;
    let owners = null;

    if (total.gt(0)) {
      owners = (
        <table className={ styles.ownerTable }>
          <tbody>
            { this.renderOwners() }
          </tbody>
        </table>
      );
    }

    return (
      <div className={ styles.body }>
        <div className={ styles.statusHeader }>
          You have { total.toFormat(0) } tokens created by your accounts
        </div>
        { owners }
      </div>
    );
  }

  renderOwners () {
    const { tokens } = this.state;

    return Object.keys(tokens).map((address) => (
      <Owner
        key={ address }
        tokens={ tokens[address] }
        address={ address }
      />
    ));
  }

  loadOwners () {
    const { accounts } = this.context;
    const addresses = Object.keys(accounts);

    loadOwnedTokens(addresses)
      .then(({ tokens, total }) => {
        this.setState({ tokens, total, loading: false });
      });
  }
}
