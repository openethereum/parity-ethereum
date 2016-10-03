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

import { loadBalances } from '../services';
import Container from '../Container';

import styles from './send.css';

export default class Send extends Component {
  static contextTypes = {
    accounts: PropTypes.object.isRequired
  }

  state = {
    loading: true
  }

  componentDidMount () {
    this.loadBalances();
  }

  render () {
    const { loading } = this.state;

    return (
      <Container center>
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
    return 'loaded';
  }

  loadBalances () {
    const { accounts } = this.context;
    const myAccounts = Object
      .values(accounts)
      .filter((account) => account.uuid)
      .map((account) => account.address);

    loadBalances(myAccounts)
      .then((balances) => {
        console.log(balances);
        this.setState({ balances, loading: false });
      });
  }
}
