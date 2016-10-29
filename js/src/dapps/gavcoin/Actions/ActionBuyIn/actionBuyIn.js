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

import { Dialog, FlatButton, TextField } from 'material-ui';

import { api } from '../../parity';
import AccountSelector from '../../AccountSelector';
import { ERRORS, validateAccount, validatePositiveNumber } from '../validation';

import styles from '../actions.css';

const NAME_ID = ' ';

export default class ActionBuyIn extends Component {
  static contextTypes = {
    instance: PropTypes.object.isRequired
  }

  static propTypes = {
    accounts: PropTypes.array,
    price: PropTypes.object,
    onClose: PropTypes.func
  }

  state = {
    account: {},
    accountError: ERRORS.invalidAccount,
    amount: 0,
    amountError: ERRORS.invalidAmount,
    maxPrice: api.util.fromWei(this.props.price.mul(1.2)).toFixed(0),
    maxPriceError: null,
    sending: false,
    complete: false
  }

  render () {
    const { complete } = this.state;

    if (complete) {
      return null;
    }

    return (
      <Dialog
        title='buy coins for a specific account'
        modal open
        className={ styles.dialog }
        actions={ this.renderActions() }>
        { this.renderFields() }
      </Dialog>
    );
  }

  renderActions () {
    const { complete } = this.state;

    if (complete) {
      return (
        <FlatButton
          className={ styles.dlgbtn }
          label='Done'
          primary
          onTouchTap={ this.props.onClose } />
      );
    }

    const { accountError, amountError, maxPriceError, sending } = this.state;
    const hasError = !!(amountError || accountError || maxPriceError);

    return ([
      <FlatButton
        className={ styles.dlgbtn }
        label='Cancel'
        primary
        onTouchTap={ this.props.onClose } />,
      <FlatButton
        className={ styles.dlgbtn }
        label='Buy'
        primary
        disabled={ hasError || sending }
        onTouchTap={ this.onSend } />
    ]);
  }

  renderFields () {
    const maxPriceLabel = `maximum price in ETH (current ${api.util.fromWei(this.props.price).toFormat(3)})`;

    return (
      <div>
        <AccountSelector
          accounts={ this.props.accounts }
          account={ this.state.account }
          errorText={ this.state.accountError }
          floatingLabelText='from account'
          hintText='the account the transaction will be made from'
          onSelect={ this.onChangeAddress } />
        <TextField
          autoComplete='off'
          floatingLabelFixed
          floatingLabelText='amount in ETH'
          fullWidth
          hintText='the amount of ETH you wish to spend'
          errorText={ this.state.amountError }
          name={ NAME_ID }
          id={ NAME_ID }
          value={ this.state.amount }
          onChange={ this.onChangeAmount } />
        <TextField
          autoComplete='off'
          floatingLabelFixed
          floatingLabelText={ maxPriceLabel }
          fullWidth
          hintText='the maxium price allowed for buying'
          errorText={ this.state.maxPriceError }
          name={ NAME_ID }
          id={ NAME_ID }
          value={ this.state.maxPrice }
          onChange={ this.onChangeMaxPrice } />
      </div>
    );
  }

  onChangeAddress = (account) => {
    this.setState({
      account,
      accountError: validateAccount(account)
    }, this.validateTotal);
  }

  onChangeAmount = (event, amount) => {
    this.setState({
      amount,
      amountError: validatePositiveNumber(amount)
    }, this.validateTotal);
  }

  onChangeMaxPrice = (event, maxPrice) => {
    this.setState({
      maxPrice,
      maxPriceError: validatePositiveNumber(maxPrice)
    });
  }

  validateTotal = () => {
    const { account, accountError, amount, amountError } = this.state;

    if (accountError || amountError) {
      return;
    }

    if (new BigNumber(amount).gt(account.ethBalance.replace(',', ''))) {
      this.setState({
        amountError: ERRORS.invalidTotal
      });
    }
  }

  onSend = () => {
    const { instance } = this.context;
    const maxPrice = api.util.toWei(this.state.maxPrice);
    const values = [this.state.account.address, maxPrice.toString()];
    const options = {
      from: this.state.account.address,
      value: api.util.toWei(this.state.amount).toString()
    };

    this.setState({
      sending: true
    });

    instance.buyin
      .estimateGas(options, values)
      .then((gasEstimate) => {
        options.gas = gasEstimate.mul(1.2).toFixed(0);
        console.log(`buyin: gas estimated as ${gasEstimate.toFixed(0)} setting to ${options.gas}`);

        return instance.buyin.postTransaction(options, values);
      })
      .then(() => {
        this.props.onClose();
        this.setState({
          sending: false,
          complete: true
        });
      })
      .catch((error) => {
        console.error('error', error);
        this.setState({
          sending: false
        });
      });
  }
}
