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
import AddressSelect from '../AddressSelect';
import Container from '../Container';

import styles from './send.css';

export default class Send extends Component {
  static contextTypes = {
    accounts: PropTypes.object.isRequired
  }

  state = {
    loading: true,
    tokens: null,
    selectedToken: null,
    availableBalances: [],
    fromAddress: null
  }

  componentDidMount () {
    this.loadBalances();
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
        Loading available tokens
      </div>
    );
  }

  renderBody () {
    const { availableBalances } = this.state;
    const fromAddresses = availableBalances.map((balance) => balance.address);

    return (
      <div className={ styles.form }>
        <div className={ styles.input }>
          <label>token type</label>
          <select onChange={ this.onSelectToken }>
            { this.renderTokens() }
          </select>
          <div className={ styles.hint }>
            The token type to transfer from
          </div>
        </div>
        <div className={ styles.input }>
          <label>transfer from</label>
          <AddressSelect
            addresses={ fromAddresses }
            onChange={ this.onSelectFrom } />
          <div className={ styles.hint }>
            The account to transfer from
          </div>
        </div>
      </div>
    );
  }

  renderTokens () {
    const { tokens } = this.state;

    return tokens.map((token) => (
      <option
        key={ token.address }
        value={ token.address }>
        { token.coin.tla } { token.coin.name }
      </option>
    ));
  }

  onSelectFrom = (event) => {
    const fromAddress = event.target.value;

    this.setState({ fromAddress });
  }

  onSelectToken = (event) => {
    const { tokens } = this.state;
    const address = event.target.value;
    const selectedToken = tokens.find((_token) => _token.address === address);
    const availableBalances = selectedToken.balances.filter((balance) => balance.balance.gt(0));

    this.setState({ selectedToken, availableBalances });
    this.onSelectFrom({ target: { value: availableBalances.address } });
  }

  loadBalances () {
    const { accounts } = this.context;
    const myAccounts = Object
      .values(accounts)
      .filter((account) => account.uuid)
      .map((account) => account.address);

    loadBalances(myAccounts)
      .then((_tokens) => {
        const tokens = _tokens.filter((token) => {
          for (let index = 0; index < token.balances.length; index++) {
            if (token.balances[index].balance.gt(0)) {
              return true;
            }
          }

          return false;
        });

        this.setState({ tokens, loading: false });
        this.onSelectToken({ target: { value: tokens[0].address } });
      });
  }
}
