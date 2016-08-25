import BigNumber from 'bignumber.js';
import React, { Component, PropTypes } from 'react';

import { Dialog, FlatButton, TextField } from 'material-ui';

import AccountSelector from '../../AccountSelector';
import AccountTextField from '../../AccountTextField';
import { renderComplete } from '../render';
import { ERRORS, validateAccount, validatePositiveNumber } from '../validation';

const { Api } = window.parity;

const DIVISOR = 10 ** 6;
const NAME_ID = ' ';

export default class ActionTransfer extends Component {
  static contextTypes = {
    instance: PropTypes.object
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
    toAccountError: ERRORS.invalidAccount,
    complete: false,
    sending: false,
    amount: 0,
    amountError: ERRORS.invalidAmount
  }

  render () {
    return (
      <Dialog
        title='Transfer'
        modal open
        actions={ this.renderActions() }>
        { this.state.complete ? renderComplete() : this.renderFields() }
      </Dialog>
    );
  }

  renderActions () {
    if (this.state.complete) {
      return (
        <FlatButton
          label='Done'
          primary
          onTouchTap={ this.props.onClose } />
      );
    }

    const hasError = !!(true || this.state.priceError || this.state.amountError || this.state.accountError);

    return ([
      <FlatButton
        label='Cancel'
        primary
        onTouchTap={ this.props.onClose } />,
      <FlatButton
        label='Transfer'
        primary
        disabled={ hasError || this.state.sending }
        onTouchTap={ this.onSend } />
    ]);
  }

  renderFields () {
    return (
      <div>
        <AccountSelector
          gavBalance
          accounts={ this.props.accounts }
          account={ this.state.fromAccount }
          accountError={ this.state.fromAccountError }
          onSelect={ this.onChangeFromAccount } />
        <AccountTextField
          accounts={ this.props.accounts }
          account={ this.state.toAccount }
          accountError={ this.state.toAccountError }
          onSelect={ this.onChangeToAccount } />
        <TextField
          autoComplete='off'
          floatingLabelFixed
          floatingLabelText='number of coins'
          fullWidth
          hintText='the number of coins to exchange for an ΞTH refund'
          errorText={ this.state.amountError }
          name={ NAME_ID }
          id={ NAME_ID }
          value={ this.state.amount }
          onChange={ this.onChangeAmount } />
      </div>
    );
  }

  onChangeFromAccount = (fromAccount) => {
    this.setState({
      fromAccount,
      fromAccountError: validateAccount(fromAccount)
    });
  }

  onChangeToAccount = (toAccount) => {
    this.setState({
      toAccount,
      toAccountError: validateAccount(toAccount)
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
    const price = Api.format.toWei(this.state.price);
    const amount = new BigNumber(this.state.amount).mul(DIVISOR);
    const values = [price.toString(), amount.toFixed(0)];
    const options = {
      from: this.state.account.address
    };

    this.setState({
      sending: true
    });

    instance.transfer
      .estimateGas(options, values)
      .then((gasEstimate) => {
        options.gas = gasEstimate.mul(1.2).toFixed(0);

        return instance.transfer.postTransaction(options, values);
      })
      .then(() => {
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
