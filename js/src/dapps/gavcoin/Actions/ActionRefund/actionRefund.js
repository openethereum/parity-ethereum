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

const DIVISOR = 10 ** 6;
const NAME_ID = ' ';

export default class ActionRefund extends Component {
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
    complete: false,
    sending: false,
    amount: 0,
    amountError: ERRORS.invalidAmount,
    price: api.util.fromWei(this.props.price).toString(),
    priceError: null
  }

  render () {
    const { complete } = this.state;

    if (complete) {
      return null;
    }

    return (
      <Dialog
        title='return coins for a refund'
        modal open
        className={ styles.dialog }
        actions={ this.renderActions() }>
        { this.renderFields() }
      </Dialog>
    );
  }

  renderActions () {
    if (this.state.complete) {
      return (
        <FlatButton
          className={ styles.dlgbtn }
          label='Done'
          primary
          onTouchTap={ this.props.onClose } />
      );
    }

    const hasError = !!(this.state.priceError || this.state.amountError || this.state.accountError);

    return ([
      <FlatButton
        className={ styles.dlgbtn }
        label='Cancel'
        primary
        onTouchTap={ this.props.onClose } />,
      <FlatButton
        className={ styles.dlgbtn }
        label='Refund'
        primary
        disabled={ hasError || this.state.sending }
        onTouchTap={ this.onSend } />
    ]);
  }

  renderFields () {
    const priceLabel = `price in ETH (current ${api.util.fromWei(this.props.price).toFormat(3)})`;

    return (
      <div>
        <AccountSelector
          gavBalance
          accounts={ this.props.accounts }
          account={ this.state.account }
          errorText={ this.state.accountError }
          floatingLabelText='from account'
          hintText='the account the transaction will be made from'
          onSelect={ this.onChangeAddress } />
        <TextField
          autoComplete='off'
          floatingLabelFixed
          floatingLabelText='number of coins'
          fullWidth
          hintText='the number of coins to exchange for an ETH refund'
          errorText={ this.state.amountError }
          name={ NAME_ID }
          id={ NAME_ID }
          value={ this.state.amount }
          onChange={ this.onChangeAmount } />
        <TextField
          autoComplete='off'
          floatingLabelFixed
          floatingLabelText={ priceLabel }
          fullWidth
          hintText='the price the refund is requested at'
          errorText={ this.state.priceError }
          name={ NAME_ID }
          id={ NAME_ID }
          value={ this.state.price }
          onChange={ this.onChangePrice } />
      </div>
    );
  }

  onChangeAddress = (account) => {
    this.setState({
      account,
      accountError: validateAccount(account)
    });
  }

  onChangeAmount = (event, amount) => {
    this.setState({
      amount,
      amountError: validatePositiveNumber(amount)
    });
  }

  onChangePrice = (event, price) => {
    this.setState({
      price,
      priceError: validatePositiveNumber(price)
    });
  }

  onSend = () => {
    const { instance } = this.context;
    const price = api.util.toWei(this.state.price);
    const amount = new BigNumber(this.state.amount).mul(DIVISOR);
    const values = [price.toString(), amount.toFixed(0)];
    const options = {
      from: this.state.account.address
    };

    this.setState({
      sending: true
    });

    instance.refund
      .estimateGas(options, values)
      .then((gasEstimate) => {
        options.gas = gasEstimate.mul(1.2).toFixed(0);
        console.log(`refund: gas estimated as ${gasEstimate.toFixed(0)} setting to ${options.gas}`);

        return instance.refund.postTransaction(options, values);
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
