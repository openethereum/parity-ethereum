import React, { Component, PropTypes } from 'react';

import Form, { Input } from '../../../Form';

import styles from '../style.css';

export default class Verify extends Component {
  static propTypes = {
    address: PropTypes.string,
    recipient: PropTypes.string,
    signer: PropTypes.bool,
    amount: PropTypes.number,
    amountTotal: PropTypes.number
  }

  state = {
    password: '',
    errorPassword: ''
  }

  render () {
    const info = this.props.signer
      ? 'Please verify the transaction information below, once it is posted you can authorise it via the Parity Signer Chrome extension'
      : 'Please verify the transaction information below and confirm the transaction with your account password';
    return (
      <Form>
        <div className={ styles.info }>
          { info }
        </div>
        <Input
          disabled
          label='account address'
          hint='the account address'
          value={ this.props.address } />
        <Input
          disabled
          label='recipient address'
          hint='the recipient address'
          value={ this.props.recipient } />
        <div className={ styles.columns }>
          <div>
            <Input
              disabled
              label='amount to transfer (in ΞTH)'
              hint='the amount to transfer to the recipient'
              value={ this.props.amount } />
          </div>
          <div>
            <Input
              disabled
              label='total transaction amount (in ΞTH)'
              hint='the amount used by this transaction'
              value={ this.props.amountTotal } />
          </div>
        </div>
        <div className={ styles.columns }>
          <div>
            <Input
              error={ this.state.errorPassword }
              label='password'
              hint='password for the origin account'
              value={ this.state.password }
              onChange={ this.onEditPassword }
              type='password' />
          </div>
        </div>
      </Form>
    );
  }

  updateParent = () => {
  }

  onEditPassword = (event) => {
    const value = event.target.value;

    this.setState({
      password: value
    }, this.updateParent);
  }
}
