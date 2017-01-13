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
import { IconButton } from 'material-ui';
import { RadioButton, RadioButtonGroup } from 'material-ui/RadioButton';
import ActionAutorenew from 'material-ui/svg-icons/action/autorenew';

import { Form, Input, IdentityIcon } from '~/ui';

import ERRORS from '../errors';

import styles from '../createAccount.css';

export default class CreateAccount extends Component {
  static contextTypes = {
    api: PropTypes.object.isRequired,
    store: PropTypes.object.isRequired
  }

  static propTypes = {
    onChange: PropTypes.func.isRequired
  }

  state = {
    accountName: '',
    accountNameError: ERRORS.noName,
    accounts: null,
    isValidName: false,
    isValidPass: true,
    passwordHint: '',
    password1: '',
    password1Error: null,
    password2: '',
    password2Error: null,
    selectedAddress: ''
  }

  componentWillMount () {
    this.createIdentities();
    this.props.onChange(false, {});
  }

  render () {
    const { accountName, accountNameError, passwordHint, password1, password1Error, password2, password2Error } = this.state;

    return (
      <Form>
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
        </div>
        { this.renderIdentitySelector() }
        { this.renderIdentities() }
      </Form>
    );
  }

  renderIdentitySelector () {
    const { accounts, selectedAddress } = this.state;

    if (!accounts) {
      return null;
    }

    const buttons = Object.keys(accounts).map((address) => {
      return (
        <RadioButton
          className={ styles.button }
          key={ address }
          value={ address } />
      );
    });

    return (
      <RadioButtonGroup
        valueSelected={ selectedAddress }
        className={ styles.selector }
        name='identitySelector'
        onChange={ this.onChangeIdentity }>
        { buttons }
      </RadioButtonGroup>
    );
  }

  renderIdentities () {
    const { accounts } = this.state;

    if (!accounts) {
      return null;
    }

    const identities = Object.keys(accounts).map((address) => {
      return (
        <div
          className={ styles.identity }
          key={ address }
          onTouchTap={ this.onChangeIdentity }>
          <IdentityIcon
            address={ address }
            center />
        </div>
      );
    });

    return (
      <div className={ styles.identities }>
        { identities }
        <div className={ styles.refresh }>
          <IconButton
            onTouchTap={ this.createIdentities }>
            <ActionAutorenew
              color='rgb(0, 151, 167)' />
          </IconButton>
        </div>
      </div>
    );
  }

  createIdentities = () => {
    const { api } = this.context;

    Promise
      .all([
        api.parity.generateSecretPhrase(),
        api.parity.generateSecretPhrase(),
        api.parity.generateSecretPhrase(),
        api.parity.generateSecretPhrase(),
        api.parity.generateSecretPhrase()
      ])
      .then((phrases) => {
        return Promise
          .all(phrases.map((phrase) => api.parity.phraseToAddress(phrase)))
          .then((addresses) => {
            const accounts = {};

            phrases.forEach((phrase, idx) => {
              accounts[addresses[idx]] = {
                address: addresses[idx],
                phrase: phrase
              };
            });

            this.setState({
              selectedAddress: addresses[0],
              accounts: accounts
            });
          });
      })
      .catch((error) => {
        console.error('createIdentities', error);
        setTimeout(this.createIdentities, 1000);
        this.newError(error);
      });
  }

  updateParent = () => {
    const { isValidName, isValidPass, accounts, accountName, passwordHint, password1, selectedAddress } = this.state;
    const isValid = isValidName && isValidPass;

    this.props.onChange(isValid, {
      address: selectedAddress,
      name: accountName,
      passwordHint,
      password: password1,
      phrase: accounts[selectedAddress].phrase
    });
  }

  onChangeIdentity = (event) => {
    const address = event.target.value || event.target.getAttribute('value');

    if (!address) {
      return;
    }

    this.setState({
      selectedAddress: address
    }, this.updateParent);
  }

  onEditPasswordHint = (event, passwordHint) => {
    this.setState({
      passwordHint
    });
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

  newError = (error) => {
    const { store } = this.context;

    store.dispatch({ type: 'newError', error });
  }
}
