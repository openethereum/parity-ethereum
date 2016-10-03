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

import BigNumber from 'bignumber.js';
import React, { Component, PropTypes } from 'react';

import Container from '../Container';
import Owner from './Owner';

export default class Overview extends Component {
  static contextTypes = {
    accounts: PropTypes.array.isRequired,
    managerInstance: PropTypes.object.isRequired
  }

  state = {
    total: new BigNumber(0),
    tokenOwners: []
  }

  componentDidMount () {
    this.loadOwners();
  }

  render () {
    const { total } = this.state;

    return (
      <Container center>
        You have { total.toFormat(0) } tokens created by your accounts
        { this.renderOwners() }
      </Container>
    );
  }

  renderOwners () {
    const { tokenOwners } = this.state;

    return tokenOwners.map((account) => (
      <Owner
        key={ account.address }
        address={ account.address } />
    ));
  }

  loadOwners () {
    const { accounts, managerInstance } = this.context;

    Promise
      .all(accounts.map((account) => managerInstance.countByOwner.call({}, [account.address])))
      .then((counts) => {
        let total = 0;
        const tokenOwners = accounts.filter((account, index) => {
          if (counts[index].gt(0)) {
            total = counts[index].add(total);
            return true;
          }
        });

        this.setState({ tokenOwners, total });
      });
  }
}
