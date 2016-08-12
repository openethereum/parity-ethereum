import React, { Component, PropTypes } from 'react';
import BigNumber from 'bignumber.js';
import { Checkbox, TextField } from 'material-ui';

import Api from '../../../../api';
import Form from '../../../Form';

import styles from './style.css';

const DEFAULT_GAS = '21000';

const CHECK_STYLE = {
  position: 'absolute',
  bottom: '8px',
  left: '1em'
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
    amount: 0.0,
    amountFull: false,
    amountGas: DEFAULT_GAS,
    amountTotal: 0.0,
    gasprice: 0,
    isValid: false
  }

  componentDidMount () {
    this.getDefaults();
  }

  render () {
    return (
      <Form>
        <TextField
          autoComplete='off'
          floatingLabelText='recipient address'
          fullWidth
          hintText='the recipient address'
          value={ this.state.recipient }
          onChange={ this.onEditRecipient } />
        <div className={ styles.columns }>
          <div>
            <TextField
              autoComplete='off'
              disabled={ this.state.amountFull }
              floatingLabelText='amount to transfer (in ΞTH)'
              fullWidth
              hintText='the amount to transfer to the recipient'
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
            <TextField
              autoComplete='off'
              floatingLabelText='gas amount'
              fullWidth
              hintText='the amount of gas to use for the transaction'
              value={ this.state.amountGas }
              onChange={ this.onEditGas } />
          </div>
        </div>
        <div className={ styles.columns }>
          <div>
            <TextField
              autoComplete='off'
              disabled
              floatingLabelText='total amount'
              fullWidth
              hintText='the total amount of the transaction'
              value={ `${this.state.amountTotal} ΞTH` } />
          </div>
        </div>
      </Form>
    );
  }

  updateParent = () => {
    this.props.onChange(this.state.isValid);
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
    const value = event.target.value;

    this.setState({
      recipient: value,
      isValid: false
    }, this.calculateTotals);
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
