import BigNumber from 'bignumber.js';
import React, { Component, PropTypes } from 'react';

import { Dialog, FlatButton, MenuItem, SelectField, TextField } from 'material-ui';

const { IdentityIcon } = window.parity.react;
const { Api } = window.parity;

const DIVISOR = 10 ** 6;
const NAME_ID = ' ';
const ERRORS = {
  invalidAccount: 'please select an account to transact from',
  invalidAmount: 'please enter a positive amount > 0'
};

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
    maxPrice: this.props.price.mul(1.2).toString(),
    maxPriceError: null
  }

  render () {
    return (
      <Dialog
        title='Buy In'
        modal open
        actions={ this.renderActions() }>
        { this.renderFields() }
      </Dialog>
    );
  }

  renderActions () {
    const hasError = !!(this.state.amountError || this.state.accountError || this.state.maxPriceError);

    return ([
      <FlatButton
        label='Cancel'
        primary
        onTouchTap={ this.props.onClose } />,
      <FlatButton
        label='Buy GAVcoin'
        primary
        disabled={ hasError }
        onTouchTap={ this.onSend } />
    ]);
  }

  renderFields () {
    const maxPriceLabel = `maxium price (current ${this.props.price.toFormat()})`;

    return (
      <div>
        { this.renderAddressSelect() }
        <TextField
          autoComplete='off'
          floatingLabelFixed
          floatingLabelText='amount (in ΞTH)'
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

  renderAddressSelect () {
    const { accounts } = this.props;
    const items = accounts.map((account) => {
      const icon = (
        <IdentityIcon inline center address={ account.address } />
      );
      const label = (
        <div className='selectaccount'>
          { icon }
          <div className='details'>
            <div className='name'>{ account.name }</div>
            <div className='balance'>{ account.ethBalance }ΞTH</div>
          </div>
        </div>
      );

      return (
        <MenuItem
          key={ account.address }
          primaryText={ account.name }
          value={ account.address }
          label={ label }
          leftIcon={ icon } />
      );
    });

    return (
      <SelectField
        autoComplete='off'
        floatingLabelFixed
        floatingLabelText='transaction account'
        fullWidth
        hintText='the account the transaction will be made from'
        errorText={ this.state.accountError }
        name={ NAME_ID }
        id={ NAME_ID }
        value={ this.state.account.address }
        onChange={ this.onChangeAddress }>
        { items }
      </SelectField>
    );
  }

  onChangeAddress = (event, idx) => {
    const { accounts } = this.props;
    const accountError = (idx >= 0 && idx < accounts.length)
      ? null
      : ERRORS.invalidAccount;

    this.setState({
      account: accounts[idx],
      accountError
    });
  }

  _isPositiveNumber (value) {
    let bn = null;

    try {
      bn = new BigNumber(value);
    } catch (e) {
    }

    if (!bn || !bn.gt(0)) {
      return ERRORS.invalidAmount;
    }

    return null;
  }

  onChangeAmount = (event, amount) => {
    const amountError = this._isPositiveNumber(amount);

    this.setState({
      amount,
      amountError
    });
  }

  onChangeMaxPrice = (event, maxPrice) => {
    const maxPriceError = this._isPositiveNumber(maxPrice);

    this.setState({
      maxPrice,
      maxPriceError
    });
  }

  onSend = () => {
    const { instance } = this.context;
    const values = [this.state.account.address, new BigNumber(this.state.maxPrice).mul(DIVISOR).toString()];
    const options = {
      from: this.state.account.address,
      value: Api.format.toWei(this.state.amount).toString()
    };

    instance.buyin
      .estimateGas(options, values)
      .then((gasEstimate) => {
        options.gas = gasEstimate.mul(1.2).toFixed(0);

        return instance.buyin.postTransaction(options, values);
      })
      .then(() => {
        console.log('success');
      })
      .catch((error) => {
        console.error('error', error);
      });
  }
}
