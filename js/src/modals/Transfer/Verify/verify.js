import React, { Component, PropTypes } from 'react';

import Api from '../../../api';
import Form, { Input } from '../../../ui/Form';

import styles from '../style.css';

const ERRORS = {
  noPassword: 'supply a valid password to confirm the transaction'
};

export default class Verify extends Component {
  static propTypes = {
    address: PropTypes.string,
    recipient: PropTypes.string,
    signer: PropTypes.bool,
    amount: PropTypes.string,
    total: PropTypes.string,
    onChange: PropTypes.func.isRequired
  }

  state = {
    password: '',
    passwordError: ERRORS.noPassword
  }

  componentDidMount () {
    this.updateParent();
  }

  render () {
    return (
      <Form>
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
              label='amount to transfer'
              hint='the amount to transfer to the recipient'
              value={ `${Api.format.fromWei(this.props.amount).toFormat()} ΞTH` } />
          </div>
          <div>
            <Input
              disabled
              label='total transaction amount'
              hint='the amount used by this transaction'
              value={ `${Api.format.fromWei(this.props.total).toFormat()} ΞTH` } />
          </div>
        </div>
        <div className={ styles.columns }>
          <div>
            <Input
              error={ this.state.passwordError }
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
    const isValid = !this.state.passwordError;

    this.props.onChange(isValid, {
      password: this.state.password
    });
  }

  onEditPassword = (event) => {
    let error = null;
    const value = event.target.value;

    if (!value || !value.length) {
      error = ERRORS.noPassword;
    }

    this.setState({
      password: value,
      passwordError: error
    }, this.updateParent);
  }
}
