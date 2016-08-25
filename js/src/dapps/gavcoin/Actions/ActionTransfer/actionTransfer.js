import BigNumber from 'bignumber.js';
import React, { Component, PropTypes } from 'react';

import { Dialog, FlatButton, TextField } from 'material-ui';

import AccountSelector from '../../AccountSelector';
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
    account: {},
    accountError: ERRORS.invalidAccount,
    complete: false,
    sending: false,
    amount: 0,
    amountError: ERRORS.invalidAmount,
    price: Api.format.fromWei(this.props.price).toString(),
    priceError: null
  }

  render () {
    return (
      <Dialog
        title='Refund'
        modal open
        actions={ this.renderActions() }>
        { this.state.complete ? this.renderComplete() : this.renderFields() }
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

    const hasError = !!(this.state.priceError || this.state.amountError || this.state.accountError);

    return ([
      <FlatButton
        label='Cancel'
        primary
        onTouchTap={ this.props.onClose } />,
      <FlatButton
        label='Refund'
        primary
        disabled={ hasError || this.state.sending }
        onTouchTap={ this.onSend } />
    ]);
  }

  renderComplete () {
    return (
      <div>Your transaction has been sent. Please visit the <a href='http://127.0.0.1:8180/' className='link' target='_blank'>Parity Signer</a> to authenticate the transfer.</div>
    );
  }

  renderFields () {
    const priceLabel = `price in ΞTH (current ${Api.format.fromWei(this.props.price).toFormat(3)})`;
    return (
      <div>
        <AccountSelector
          gavBalance
          accounts={ this.props.accounts }
          account={ this.state.account }
          accountError={ this.state.accountError }
          onSelect={ this.onChangeAddress } />
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
    const price = Api.format.toWei(this.state.price);
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

        return instance.refund.postTransaction(options, values);
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
