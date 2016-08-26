import React, { Component, PropTypes } from 'react';

import { Dialog, FlatButton, TextField } from 'material-ui';

import AccountSelector from '../../AccountSelector';
import StepComplete from '../StepComplete';
import { ERRORS, validateAccount, validatePositiveNumber } from '../validation';

import styles from '../style.css';

const { Api } = window.parity;

const NAME_ID = ' ';

export default class ActionBuyIn extends Component {
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
    amount: 0,
    amountError: ERRORS.invalidAmount,
    maxPrice: Api.format.fromWei(this.props.price.mul(1.2)).toString(),
    maxPriceError: null,
    sending: false,
    complete: false
  }

  render () {
    return (
      <Dialog
        title='buy coins for a specific account'
        modal open
        className={ styles.dialog }
        actions={ this.renderActions() }>
        { this.state.complete ? <StepComplete /> : this.renderFields() }
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

    const hasError = !!(this.state.amountError || this.state.accountError || this.state.maxPriceError);

    return ([
      <FlatButton
        label='Cancel'
        primary
        onTouchTap={ this.props.onClose } />,
      <FlatButton
        label='Buy'
        primary
        disabled={ hasError || this.state.sending }
        onTouchTap={ this.onSend } />
    ]);
  }

  renderFields () {
    const maxPriceLabel = `maximum price in ΞTH (current ${Api.format.fromWei(this.props.price).toFormat(3)})`;

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
          floatingLabelText='amount in ΞTH'
          fullWidth
          hintText='the amount of ΞTH you wish to spend'
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
    });
  }

  onChangeAmount = (event, amount) => {
    this.setState({
      amount,
      amountError: validatePositiveNumber(amount)
    });
  }

  onChangeMaxPrice = (event, maxPrice) => {
    this.setState({
      maxPrice,
      maxPriceError: validatePositiveNumber(maxPrice)
    });
  }

  onSend = () => {
    const maxPrice = Api.format.toWei(this.state.maxPrice);
    const { instance } = this.context;
    const values = [this.state.account.address, maxPrice.toString()];
    const options = {
      from: this.state.account.address,
      value: Api.format.toWei(this.state.amount).toString()
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
