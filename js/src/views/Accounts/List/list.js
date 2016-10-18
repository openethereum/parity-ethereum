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

import { Container } from '../../../ui';

import Summary from '../Summary';
import styles from './list.css';

export default class List extends Component {
  static propTypes = {
    accounts: PropTypes.object,
    balances: PropTypes.object,
    link: PropTypes.string,
    search: PropTypes.array,
    empty: PropTypes.bool,
    handleAddSearchToken: PropTypes.func
  };

  render () {
    return (
      <div className={ styles.list }>
        { this.renderAccounts() }
      </div>
    );
  }

  renderAccounts () {
    const { accounts, balances, link, empty, handleAddSearchToken } = this.props;

    if (empty) {
      return (
        <Container className={ styles.empty }>
          <div>
            There are currently no accounts or addresses to display.
          </div>
        </Container>
      );
    }

    const addresses = this.getFilteredAddresses();

    return addresses.map((address, idx) => {
      const account = accounts[address] || {};
      const balance = balances[address] || {};

      return (
        <div
          className={ styles.item }
          key={ address }>
          <Summary
            link={ link }
            account={ account }
            balance={ balance }
            handleAddSearchToken={ handleAddSearchToken } />
        </div>
      );
    });
  }

  getFilteredAddresses () {
    const { accounts, search } = this.props;
    const searchValues = (search || []).map(v => v.toLowerCase());

    if (searchValues.length === 0) {
      return Object.keys(accounts);
    }

    return Object.keys(accounts)
      .filter((address) => {
        const account = accounts[address];

        const tags = account.meta.tags || [];
        const name = account.name || '';

        const values = []
          .concat(tags, name)
          .map(v => v.toLowerCase());

        return values
          .filter((value) => {
            return searchValues
              .map(searchValue => value.indexOf(searchValue) >= 0)
              // `current && truth, true` => use tokens as AND
              // `current || truth, false` => use tokens as OR
              .reduce((current, truth) => current || truth, false);
          })
          .length > 0;
      });
  }
}
