// Copyright 2015, 2016 Parity Technologies (UK) Ltd.
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

import { observer } from 'mobx-react';
import React, { Component, PropTypes } from 'react';
import { FormattedMessage } from 'react-intl';

import { Form, Input } from '~/ui';

import styles from '../createAccount.css';

import ERRORS from '../errors';

@observer
export default class RawKey extends Component {
  static contextTypes = {
    api: PropTypes.object.isRequired
  }

  static propTypes = {
    onChange: PropTypes.func.isRequired,
    store: PropTypes.object.isRequired
  }

  state = {
    accountName: '',
    accountNameError: ERRORS.noName,
    isValidKey: false,
    isValidName: false,
    isValidPass: false,
    passwordHint: '',
    password1: '',
    password1Error: null,
    password2: '',
    password2Error: null,
    rawKey: '',
    rawKeyError: ERRORS.noKey
  }

  componentWillMount () {
    this.props.onChange(false, {});
  }

  render () {
    const { accountName, accountNameError, passwordHint, password1, password1Error, password2, password2Error, rawKey, rawKeyError } = this.state;

    return (
      <Form>
        <Input
          error={ rawKeyError }
          hint={
            <FormattedMessage
              id='createAccount.rawKey.private.hint'
              defaultMessage='the raw hex encoded private key' />
          }
          label={
            <FormattedMessage
              id='createAccount.rawKey.private.label'
              defaultMessage='private key' />
          }
          onChange={ this.onEditKey }
          value={ rawKey } />
        <Input
          error={ accountNameError }
          hint={
            <FormattedMessage
              id='createAccount.rawKey.name.hint'
              defaultMessage='a descriptive name for the account' />
          }
          label={
            <FormattedMessage
              id='createAccount.rawKey.name.label'
              defaultMessage='account name' />
          }
          onChange={ this.onEditAccountName }
          value={ accountName } />
        <Input
          hint={
            <FormattedMessage
              id='createAccount.rawKey.hint.hint'
              defaultMessage='(optional) a hint to help with remembering the password' />
          }
          label={
            <FormattedMessage
              id='createAccount.rawKey.hint.label'
              defaultMessage='password hint' />
          }
          onChange={ this.onEditPasswordHint }
          value={ passwordHint } />
        <div className={ styles.passwords }>
          <div className={ styles.password }>
            <Input
              error={ password1Error }
              hint={
                <FormattedMessage
                  id='createAccount.rawKey.password.hint'
                  defaultMessage='a strong, unique password' />
              }
              label={
                <FormattedMessage
                  id='createAccount.rawKey.password.label'
                  defaultMessage='password' />
              }
              onChange={ this.onEditPassword1 }
              type='password'
              value={ password1 } />
          </div>
          <div className={ styles.password }>
            <Input
              error={ password2Error }
              hint={
                <FormattedMessage
                  id='createAccount.rawKey.password2.hint'
                  defaultMessage='verify your password' />
              }
              label={
                <FormattedMessage
                  id='createAccount.rawKey.password2.label'
                  defaultMessage='password (repeat)' />
              }
              onChange={ this.onEditPassword2 }
              type='password'
              value={ password2 } />
          </div>
        </div>
      </Form>
    );
  }

  updateParent = () => {
    const { isValidName, isValidPass, isValidKey, accountName, passwordHint, password1, rawKey } = this.state;
    const isValid = isValidName && isValidPass && isValidKey;

    this.props.onChange(isValid, {
      name: accountName,
      passwordHint,
      password: password1,
      rawKey
    });
  }

  onEditPasswordHint = (event, value) => {
    this.setState({
      passwordHint: value
    });
  }

  onEditKey = (event) => {
    const { api } = this.context;
    const rawKey = event.target.value;
    let rawKeyError = null;

    if (!rawKey || !rawKey.trim().length) {
      rawKeyError = ERRORS.noKey;
    } else if (rawKey.substr(0, 2) !== '0x' || rawKey.substr(2).length !== 64 || !api.util.isHex(rawKey)) {
      rawKeyError = ERRORS.invalidKey;
    }

    this.setState({
      rawKey,
      rawKeyError,
      isValidKey: !rawKeyError
    }, this.updateParent);
  }

  onEditAccountName = (event) => {
    const accountName = event.target.value;
    let accountNameError = null;

    if (!accountName || !accountName.trim().length) {
      accountNameError = ERRORS.noName;
    }

    this.setState({
      accountName,
      accountNameError,
      isValidName: !accountNameError
    }, this.updateParent);
  }

  onEditPassword1 = (event) => {
    const password1 = event.target.value;
    let password2Error = null;

    if (password1 !== this.state.password2) {
      password2Error = ERRORS.noMatchPassword;
    }

    this.setState({
      password1,
      password1Error: null,
      password2Error,
      isValidPass: !password2Error
    }, this.updateParent);
  }

  onEditPassword2 = (event) => {
    const password2 = event.target.value;
    let password2Error = null;

    if (password2 !== this.state.password1) {
      password2Error = ERRORS.noMatchPassword;
    }

    this.setState({
      password2,
      password2Error,
      isValidPass: !password2Error
    }, this.updateParent);
  }
}
