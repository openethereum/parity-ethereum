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

import React, { Component, PropTypes } from 'react';
import { Checkbox } from 'material-ui';

import { Form, Input } from '~/ui';

import styles from '../createAccount.css';

import ERRORS from '../errors';

export default class RecoveryPhrase extends Component {
  static propTypes = {
    onChange: PropTypes.func.isRequired
  }

  state = {
    accountName: '',
    accountNameError: ERRORS.noName,
    isValidPass: true,
    isValidName: false,
    isValidPhrase: true,
    passwordHint: '',
    password1: '',
    password1Error: null,
    password2: '',
    password2Error: null,
    recoveryPhrase: '',
    recoveryPhraseError: null,
    windowsPhrase: false
  }

  componentWillMount () {
    this.props.onChange(false, {});
  }

  render () {
    const { accountName, accountNameError, passwordHint, password1, password1Error, password2, password2Error, recoveryPhrase, windowsPhrase } = this.state;

    return (
      <Form>
        <Input
          hint='the account recovery phrase'
          label='account recovery phrase'
          value={ recoveryPhrase }
          onChange={ this.onEditPhrase } />
        <Input
          label='account name'
          hint='a descriptive name for the account'
          error={ accountNameError }
          value={ accountName }
          onChange={ this.onEditAccountName } />
        <Input
          label='password hint'
          hint='(optional) a hint to help with remembering the password'
          value={ passwordHint }
          onChange={ this.onEditPasswordHint } />
        <div className={ styles.passwords }>
          <div className={ styles.password }>
            <Input
              label='password'
              hint='a strong, unique password'
              type='password'
              error={ password1Error }
              value={ password1 }
              onChange={ this.onEditPassword1 } />
          </div>
          <div className={ styles.password }>
            <Input
              label='password (repeat)'
              hint='verify your password'
              type='password'
              error={ password2Error }
              value={ password2 }
              onChange={ this.onEditPassword2 } />
          </div>
          <Checkbox
            className={ styles.checkbox }
            label='Key was created with Parity <1.4.5 on Windows'
            checked={ windowsPhrase }
            onCheck={ this.onToggleWindowsPhrase } />
        </div>
      </Form>
    );
  }

  updateParent = () => {
    const { accountName, isValidName, isValidPass, isValidPhrase, password1, passwordHint, recoveryPhrase, windowsPhrase } = this.state;
    const isValid = isValidName && isValidPass && isValidPhrase;

    this.props.onChange(isValid, {
      name: accountName,
      password: password1,
      passwordHint,
      phrase: recoveryPhrase,
      windowsPhrase
    });
  }

  onEditPasswordHint = (event, value) => {
    this.setState({
      passwordHint: value
    });
  }

  onToggleWindowsPhrase = (event) => {
    this.setState({
      windowsPhrase: !this.state.windowsPhrase
    }, this.updateParent);
  }

  onEditPhrase = (event) => {
    const recoveryPhrase = event.target.value
      .toLowerCase() // wordlists are lowercase
      .trim() // remove whitespace at both ends
      .replace(/\s/g, ' ') // replace any whitespace with single space
      .replace(/ +/g, ' '); // replace multiple spaces with a single space

    const phraseParts = recoveryPhrase
      .split(' ')
      .map((part) => part.trim())
      .filter((part) => part.length);

    this.setState({
      recoveryPhrase: phraseParts.join(' '),
      recoveryPhraseError: null,
      isValidPhrase: true
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
