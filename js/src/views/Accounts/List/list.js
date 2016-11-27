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
    order: PropTypes.string,
    orderFallback: PropTypes.string,
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

    const addresses = this.getAddresses();

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

  getAddresses () {
    const filteredAddresses = this.getFilteredAddresses();
    return this.sortAddresses(filteredAddresses);
  }

  sortAddresses (addresses) {
    const { order, orderFallback } = this.props;

    if (!order) {
      return addresses;
    }

    const { accounts } = this.props;

    return addresses.sort((addressA, addressB) => {
      const accountA = accounts[addressA];
      const accountB = accounts[addressB];

      const sort = this.compareAccounts(accountA, accountB, order);

      if (sort === 0 && orderFallback) {
        return this.compareAccounts(accountA, accountB, orderFallback);
      }

      return sort;
    });
  }

  compareAccounts (accountA, accountB, key) {
    if (key === 'name') {
      return accountA.name.localeCompare(accountB.name);
    }

    if (key === 'eth') {
      const { balances } = this.props;

      const balanceA = balances[accountA.address];
      const balanceB = balances[accountB.address];

      if (!balanceA && !balanceB) return 0;
      if (balanceA && !balanceB) return -1;
      if (!balanceA && balanceB) return 1;

      const ethA = balanceA.tokens.find(token => token.token.tag.toLowerCase() === 'eth');
      const ethB = balanceB.tokens.find(token => token.token.tag.toLowerCase() === 'eth');

      if (!ethA && !ethB) return 0;
      if (ethA && !ethB) return -1;
      if (!ethA && ethB) return 1;

      return -1 * ethA.value.comparedTo(ethB.value);
    }

    if (key === 'tags') {
      const tagsA = [].concat(accountA.meta.tags)
        .filter(t => t)
        .sort()
        .join('');

      const tagsB = [].concat(accountB.meta.tags)
        .filter(t => t)
        .sort()
        .join('');

      if (!tagsA && !tagsB) return 0;
      if (tagsA && !tagsB) return -1;
      if (!tagsA && tagsB) return 1;

      return tagsA.localeCompare(tagsB);
    }

    const reverse = key === 'timestamp' ? -1 : 1;

    const metaA = accountA.meta[key];
    const metaB = accountB.meta[key];

    if (!metaA && !metaB) {
      return 0;
    }

    if (metaA && !metaB) {
      return -1;
    }

    if (metaA < metaB) {
      return -1 * reverse;
    }

    if (!metaA && metaB) {
      return 1;
    }

    if (metaA > metaB) {
      return 1 * reverse;
    }

    return 0;
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

        return searchValues
          .map(searchValue => {
            return values
              .some(value => value.indexOf(searchValue) >= 0);
          })
          // `current && truth, true` => use tokens as AND
          // `current || truth, false` => use tokens as OR
          .reduce((current, truth) => current && truth, true);
      });
  }
}
