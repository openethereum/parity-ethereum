import React, { Component, PropTypes } from 'react';
import BigNumber from 'bignumber.js';
import { Checkbox, FloatingActionButton } from 'material-ui';

import CommunicationContacts from 'material-ui/svg-icons/communication/contacts';

import Api from '../../../api';
import AddressSelector from '../../AddressSelector';
import Form, { Input } from '../../../ui/Form';

import styles from '../style.css';

const DEFAULT_GAS = '21000';

const CHECK_STYLE = {
  position: 'absolute',
  bottom: '8px',
  left: '1em'
};

const ERRORS = {
  requireRecipient: 'a recipient network address is required for the transaction',
  invalidAddress: 'the supplied address is an invalid network address',
  invalidAmount: 'the supplied amount should be a valid positive number',
  largeAmount: 'the transaction total is higher than the available balance'
};

export default class Details extends Component {
  static contextTypes = {
    api: PropTypes.object.isRequired
  }

  static propTypes = {
    address: PropTypes.string.isRequired,
    balance: PropTypes.object,
    onChange: PropTypes.func.isRequired
  }

  state = {
    recipient: '',
    recipientError: ERRORS.requireRecipient,
    amount: 0,
    accountError: null,
    amountFull: false,
    gas: DEFAULT_GAS,
    gasError: null,
    total: 0,
    totalError: null,
    gasprice: 0,
    showAddresses: false
  }

  componentDidMount () {
    this.getDefaults();
  }

  render () {
    return (
      <Form>
        <AddressSelector
          onSelect={ this.onSelectRecipient }
          visible={ this.state.showAddresses } />
        <div>
          <Input
            label='recipient address'
            hint='the recipient address'
            error={ this.state.recipientError }
            value={ this.state.recipient }
            onChange={ this.onEditRecipient } />
          <div className={ styles.floatbutton }>
            <FloatingActionButton
              primary mini
              onTouchTap={ this.onContacts }>
              <CommunicationContacts />
            </FloatingActionButton>
          </div>
        </div>
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
              disabled
              label='total amount'
              hint='the total amount of the transaction'
              error={ this.state.totalError }
              value={ `${Api.format.fromWei(this.state.total).toFormat()} ΞTH` } />
          </div>
        </div>
      </Form>
    );
  }

  renderExtended () {
    return (
      <div className={ styles.columns }>
        <div>
          <Input
            label='gas amount'
            hint='the amount of gas to use for the transaction'
            error={ this.state.gasError }
            value={ this.state.gas }
            onChange={ this.onEditGas } />
        </div>
      </div>
    );
  }

  onSelectRecipient = (recipient) => {
    this.setState({
      showAddresses: false
    }, () => {
      this.validateRecipient(recipient);
    });
  }

  onCheckFullAmount = (event) => {
    let amount = this.state.amount;

    if (!this.state.amountFull) {
      const gas = new BigNumber(this.state.gasprice).mul(new BigNumber(this.state.gas || 0));
      const balance = new BigNumber(this.props.balance ? this.props.balance.value : 0);

      amount = Api.format.fromWei(balance.minus(gas));

      if (amount.lt(0)) {
        amount = new BigNumber(0);
      }
    }

    this.setState({
      amount: amount.toString(),
      amountFull: !this.state.amountFull
    }, this.calculateTotals);
  }

  onEditAmount = (event) => {
    let value = event.target.value;
    let error = null;
    let num = null;

    try {
      num = new BigNumber(value);
    } catch (e) {
      num = null;
    }

    if (!num || num.lt(0)) {
      error = ERRORS.invalidAmount;
    }

    this.setState({
      amount: value,
      amountError: error
    }, this.calculateTotals);
  }

  onEditGas = (event) => {
    let value = event.target.value;
    let error = null;
    let num = null;

    try {
      num = new BigNumber(value);
    } catch (e) {
      num = null;
    }

    if (!num || num.lt(0)) {
      error = ERRORS.invalidAmount;
    }

    this.setState({
      gas: value,
      gasError: error
    }, this.calculateTotals);
  }

  validateRecipient (value) {
    let error = null;

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

  onEditRecipient = (event) => {
    this.validateRecipient(event.target.value);
  }

  onContacts = () => {
    this.setState({
      showAddresses: true
    });
  }

  updateParent = () => {
    const isValid = !this.state.recipientError && !this.state.amountError && !this.state.gasError;

    this.props.onChange(isValid, {
      amount: Api.format.toWei(this.state.amount).toString(),
      gas: this.state.gas,
      recipient: this.state.recipient,
      total: this.state.total
    });
  }

  calculateTotals = () => {
    const gas = new BigNumber(this.state.gasprice).mul(new BigNumber(this.state.gas || 0));
    const amount = Api.format.toWei(this.state.amount || 0);
    const total = amount.plus(gas);
    const balance = new BigNumber(this.props.balance ? this.props.balance.value : 0);
    let error = null;

    if (total.gt(balance)) {
      error = ERRORS.largeAmount;
    }

    this.setState({
      total: total.toString(),
      totalError: error
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
