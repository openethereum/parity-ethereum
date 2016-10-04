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
    fromAddress: null,
    fromBalance: null,
    toAddress: null,
    toKnown: true,
    amount: 0,
    amountError: null
  }

  componentDidMount () {
    this.loadBalances();
    this.onAmountChange({ target: { value: '0' } });
  }

  render () {
    const { loading } = this.state;

    return loading
      ? this.renderLoading()
      : this.renderBody();
  }

  renderLoading () {
    return (
      <Container>
        <div className={ styles.statusHeader }>
          Loading available tokens
        </div>
      </Container>
    );
  }

  renderBody () {
    const { accounts } = this.context;
    const { availableBalances, fromAddress, amount, amountError, toKnown, toAddress } = this.state;
    const fromBalance = availableBalances.find((balance) => balance.address === fromAddress);
    const fromAddresses = availableBalances.map((balance) => balance.address);
    const toAddresses = Object.keys(accounts);
    const toInput = toKnown
      ? <AddressSelect addresses={ toAddresses } onChange={ this.onChangeTo } />
      : <input value={ toAddress } onChange={ this.onChangeTo } />;
    const hasError = amountError;
    const error = `${styles.input} ${styles.error}`;
    const maxAmountHint = `Value to transfer (max: ${fromBalance ? fromBalance.balance.div(1000000).toFormat(6) : '1'})`;

    return (
      <Container>
        <div className={ styles.form }>
          <div className={ styles.input }>
            <label>token type</label>
            <select onChange={ this.onSelectToken }>
              { this.renderTokens() }
            </select>
            <div className={ styles.hint }>
              The token type to transfer
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
          <div className={ styles.input }>
            <label>transfer to</label>
            <select onChange={ this.onChangeToType }>
              <option value='known'>Known, Select from list</option>
              <option value='unknown'>Unknown, Keyboard input</option>
            </select>
            <div className={ styles.hint }>
              the type of address input
            </div>
          </div>
          <div className={ styles.input }>
            <label />
            { toInput }
            <div className={ styles.hint }>
              the account to transfer to
            </div>
          </div>
          <div className={ amountError ? error : styles.input }>
            <label>amount</label>
            <input
              type='number'
              min='0'
              step='0.1'
              value={ amount }
              max={ fromBalance ? fromBalance.balance.div(1000000).toFixed(6) : 1 }
              onChange={ this.onAmountChange } />
            <div className={ styles.hint }>
              { amountError || maxAmountHint }
            </div>
          </div>
          <div className={ styles.input }>
            <label />
            <div className={ styles.buttonRow }>
              <div
                className={ styles.button }
                disabled={ hasError }
                onClick={ this.onSend }>
                Send Tokens
              </div>
            </div>
          </div>
        </div>
      </Container>
    );
  }

  renderTokens () {
    const { tokens } = this.state;

    return tokens.map((token) => (
      <option
        key={ token.address }
        value={ token.address }>
        { token.coin.tla } / { token.coin.name }
      </option>
    ));
  }

  onSelectFrom = (event) => {
    const fromAddress = event.target.value;

    this.setState({ fromAddress });
  }

  onChangeTo = (event) => {
    const toAddress = event.target.value;

    this.setState({ toAddress });
  }

  onChangeToType = (event) => {
    const toKnown = event.target.value === 'known';

    this.setState({ toKnown });
  }

  onSelectToken = (event) => {
    const { tokens } = this.state;
    const address = event.target.value;
    const selectedToken = tokens.find((_token) => _token.address === address);
    const availableBalances = selectedToken.balances.filter((balance) => balance.balance.gt(0));

    this.setState({ selectedToken, availableBalances });
    this.onSelectFrom({ target: { value: availableBalances[0].address } });
  }

  onAmountChange = (event) => {
    const amount = parseFloat(event.target.value);
    const amountError = !isFinite(amount) || amount <= 0
      ? 'amount needs to be > 0'
      : null;

    this.setState({ amount, amountError });
  }

  onSend = () => {
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
