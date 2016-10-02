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

export default class Overview extends Component {
  static contextTypes = {
    accounts: PropTypes.array.isRequired,
    managerInstance: PropTypes.object.isRequired
  }

  state = {
    total: new BigNumber(0),
    tokens: {}
  }

  componentDidMount () {
    const { accounts, managerInstance } = this.context;
    let total = 0;

    Promise
      .all(accounts.map((account) => managerInstance.countByOwner.call({}, [account.address])))
      .then((counts) => {
        return Promise
          .all(accounts.map((account, index) => {
            const promises = [];

            total = counts[index].add(total);
            for (let i = 0; i < counts[index]; i++) {
              promises.push(managerInstance.getByOwner.call({}, [account.address, i]));
            }

            return Promise.all(promises);
          }));
      })
      .then((_tokens) => {
        this.setState({
          total,
          tokens: accounts.reduce((tokens, account, index) => {
            tokens[account.address] = _tokens[index];
            return tokens;
          }, {})
        });
      });
  }

  render () {
    const { total } = this.state;

    return (
      <Container center>
        You have { total.toFormat(0) } tokens created by your accounts
      </Container>
    );
  }
}
