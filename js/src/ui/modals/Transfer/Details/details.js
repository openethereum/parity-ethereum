import React, { Component, PropTypes } from 'react';
import BigNumber from 'bignumber.js';
import { Checkbox } from 'material-ui';

import Api from '../../../../api';
import Form, { Input } from '../../../Form';

import styles from '../style.css';

const DEFAULT_GAS = '30000';

const CHECK_STYLE = {
  position: 'absolute',
  bottom: '8px',
  left: '1em'
};

const ERRORS = {
  requireRecipient: 'a recipient account is required for the transaction',
  invalidAddress: 'the supplied address is an invalid network address'
};

export default class Details extends Component {
  static contextTypes = {
    api: PropTypes.object.isRequired
  }

  static propTypes = {
    address: PropTypes.string.isRequired,
    onChange: PropTypes.func.isRequired
  }

  state = {
    recipient: '',
    recipientError: ERRORS.requireRecipient,
    amount: 0.0,
    amountFull: false,
    amountGas: DEFAULT_GAS,
    amountTotal: 0.0,
    gasprice: 0
  }

  componentDidMount () {
    this.getDefaults();
  }

  render () {
    return (
      <Form>
        <div className={ styles.info }>
          Complete the information for the transaction with a valid recipient and the amount to be transferred. For normal transactions, the gas value can be left at the default.
        </div>
        <Input
          label='recipient address'
          hint='the recipient address'
          error={ this.state.recipientError }
          value={ this.state.recipient }
          onChange={ this.onEditRecipient } />
        <div className={ styles.columns }>
          <div>
            <Input
              disabled={ this.state.amountFull }
              label='amount to transfer (in ΞTH)'
              hint='the amount to transfer to the recipient'
              value={ this.state.amount }
              onChange={ this.onEditAmount } />
          </div>
          <div>
            <Checkbox
              checked={ this.state.amountFull }
              label='full account balance'
              onCheck={ this.onCheckFullAmount }
              style={ CHECK_STYLE } />
          </div>
        </div>
        <div className={ styles.columns }>
          <div>
            <Input
              label='gas amount'
              hint='the amount of gas to use for the transaction'
              value={ this.state.amountGas }
              onChange={ this.onEditGas } />
          </div>
        </div>
        <div className={ styles.columns }>
          <div>
            <Input
              disabled
              label='total amount'
              hint='the total amount of the transaction'
              value={ `${this.state.amountTotal} ΞTH` } />
          </div>
        </div>
      </Form>
    );
  }

  onCheckFullAmount = (event) => {
    this.setState({
      amountFull: !this.state.amountFull
    });
  }

  onEditAmount = (event) => {
    const value = event.target.value;

    this.setState({
      amount: value
    }, this.calculateTotals);
  }

  onEditGas = (event) => {
    const value = event.target.value;

    this.setState({
      amount: value
    }, this.calculateTotals);
  }

  onEditRecipient = (event) => {
    let error = null;
    const value = event.target.value;

    if (!value || !value.length) {
      error = ERRORS.requireRecipient;
    } else if (!Api.format.isAddressValid(value)) {
      error = ERRORS.invalidAddress;
    }

    this.setState({
      recipient: value,
      recipientError: error
    }, this.calculateTotals);
  }

  updateParent = () => {
    const isValid = !this.state.recipientError;

    this.props.onChange(isValid, {
      recipient: this.state.recipient
    });
  }

  calculateTotals = () => {
    const gas = new BigNumber(this.state.gasprice).mul(new BigNumber(this.state.amountGas || 0));
    const amount = Api.format.toWei(this.state.amount || 0);
    const total = Api.format.fromWei(amount.plus(gas));

    this.setState({
      amountTotal: total.toNumber()
    }, this.updateParent);
  }

  getDefaults = () => {
    const api = this.context.api;

    api.eth
      .gasPrice()
      .then((gasprice) => {
        this.setState({
          gasprice: gasprice.toString()
        }, this.calculateTotals);
      });
  }
}
