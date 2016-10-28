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

import { Dialog, FlatButton, TextField, Toggle } from 'material-ui';

import AccountSelector from '../../AccountSelector';
import AccountSelectorText from '../../AccountSelectorText';
import { ERRORS, validateAccount, validatePositiveNumber } from '../validation';

import styles from '../actions.css';

const DIVISOR = 10 ** 6;
const NAME_ID = ' ';

export default class ActionTransfer extends Component {
  static contextTypes = {
    instance: PropTypes.object.isRequired
  }

  static propTypes = {
    accounts: PropTypes.array,
    price: PropTypes.object,
    onClose: PropTypes.func
  }

  state = {
    fromAccount: {},
    fromAccountError: ERRORS.invalidAccount,
    toAccount: {},
    toAccountError: ERRORS.invalidRecipient,
    inputAccount: false,
    complete: false,
    sending: false,
    amount: 0,
    amountError: ERRORS.invalidAmount
  }

  render () {
    const { complete } = this.state;

    if (complete) {
      return null;
    }

    return (
      <Dialog
        title='send coins to another account'
        modal open
        className={ styles.dialog }
        actions={ this.renderActions() }>
        { this.renderFields() }
      </Dialog>
    );
  }

  renderActions () {
    const { complete, sending, amountError, fromAccountError, toAccountError } = this.state;

    if (complete) {
      return (
        <FlatButton
          className={ styles.dlgbtn }
          label='Done'
          primary
          onTouchTap={ this.props.onClose } />
      );
    }

    const hasError = !!(amountError || fromAccountError || toAccountError);

    return ([
      <FlatButton
        className={ styles.dlgbtn }
        label='Cancel'
        primary
        onTouchTap={ this.props.onClose } />,
      <FlatButton
        className={ styles.dlgbtn }
        label='Transfer'
        primary
        disabled={ hasError || sending }
        onTouchTap={ this.onSend } />
    ]);
  }

  renderFields () {
    const { accounts } = this.props;
    const { fromAccount, fromAccountError, toAccount, toAccountError, inputAccount, amount, amountError } = this.state;

    return (
      <div>
        <AccountSelector
          gavBalance
          accounts={ accounts }
          account={ fromAccount }
          errorText={ fromAccountError }
          floatingLabelText='from account'
          hintText='the account the transaction will be made from'
          onSelect={ this.onChangeFromAccount } />
        <div className={ styles.overlay }>
          <AccountSelectorText
            gavBalance anyAccount
            selector={ !inputAccount }
            accounts={ accounts }
            account={ toAccount }
            errorText={ toAccountError }
            floatingLabelText='to account'
            hintText='the account the coins will be sent to'
            onChange={ this.onChangeToAccount } />
          <Toggle
            className={ styles.overlaytoggle }
            label='Edit'
            labelPosition='right'
            toggled={ inputAccount }
            onToggle={ this.onChangeToInput } />
        </div>
        <TextField
          autoComplete='off'
          floatingLabelFixed
          floatingLabelText='number of coins'
          fullWidth
          hintText='the number of coins to transfer'
          errorText={ amountError }
          name={ NAME_ID }
          id={ NAME_ID }
          value={ amount }
          onChange={ this.onChangeAmount } />
      </div>
    );
  }

  onChangeFromAccount = (fromAccount) => {
    this.setState({
      fromAccount,
      fromAccountError: validateAccount(fromAccount)
    }, this.validateTotal);
  }

  onChangeToAccount = (toAccount) => {
    this.setState({
      toAccount,
      toAccountError: validateAccount(toAccount)
    });
  }

  onChangeToInput = () => {
    this.setState({
      inputAccount: !this.state.inputAccount
    });
  }

  onChangeAmount = (event, amount) => {
    this.setState({
      amount,
      amountError: validatePositiveNumber(amount)
    }, this.validateTotal);
  }

  validateTotal = () => {
    const { fromAccount, fromAccountError, amount, amountError } = this.state;

    if (fromAccountError || amountError) {
      return;
    }

    if (new BigNumber(amount).gt(fromAccount.gavBalance.replace(',', ''))) {
      this.setState({
        amountError: ERRORS.invalidTotal
      });
    }
  }

  onSend = () => {
    const { instance } = this.context;
    const amount = new BigNumber(this.state.amount).mul(DIVISOR);
    const values = [this.state.toAccount.address, amount.toFixed(0)];
    const options = {
      from: this.state.fromAccount.address
    };

    this.setState({
      sending: true
    });

    instance.transfer
      .estimateGas(options, values)
      .then((gasEstimate) => {
        options.gas = gasEstimate.mul(1.2).toFixed(0);
        console.log(`transfer: gas estimated as ${gasEstimate.toFixed(0)} setting to ${options.gas}`);

        return instance.transfer.postTransaction(options, values);
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
