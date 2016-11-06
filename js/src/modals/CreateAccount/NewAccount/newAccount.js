// Copyright 2015, 2016 Ethcore (UK) Ltd.
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
import IconButton from 'material-ui/IconButton';
import { RadioButton, RadioButtonGroup } from 'material-ui/RadioButton';
import ActionAutorenew from 'material-ui/svg-icons/action/autorenew';

import { Form, Input, IdentityIcon } from '../../../ui';

import styles from '../createAccount.css';

const ERRORS = {
  noName: 'you need to specify a valid name for the account',
  noPhrase: 'you need to specify the recovery phrase',
  noKey: 'you need to provide the raw private key',
  invalidKey: 'the raw key needs to be hex, 64 characters in length and contain the prefix "0x"',
  invalidPassword: 'you need to specify a password >= 8 characters',
  noMatchPassword: 'the supplied passwords does not match'
};

export {
  ERRORS
};

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
    passwordHint: '',
    password1: '',
    password1Error: ERRORS.invalidPassword,
    password2: '',
    password2Error: ERRORS.noMatchPassword,
    accounts: null,
    selectedAddress: '',
    isValidPass: false,
    isValidName: false
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

            console.log(accounts);

            this.setState({
              selectedAddress: addresses[0],
              accounts: accounts
            });
          });
      })
      .catch((error) => {
        console.log('createIdentities', error);

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

  onEditPasswordHint = (event, value) => {
    this.setState({
      passwordHint: value
    });
  }

  onEditAccountName = (event) => {
    const value = event.target.value;
    let error = null;

    if (!value || value.trim().length < 2) {
      error = ERRORS.noName;
    }

    this.setState({
      accountName: value,
      accountNameError: error,
      isValidName: !error
    }, this.updateParent);
  }

  onEditPassword1 = (event) => {
    const value = event.target.value;
    let error1 = null;
    let error2 = null;

    if (!value || value.trim().length < 8) {
      error1 = ERRORS.invalidPassword;
    }

    if (value !== this.state.password2) {
      error2 = ERRORS.noMatchPassword;
    }

    this.setState({
      password1: value,
      password1Error: error1,
      password2Error: error2,
      isValidPass: !error1 && !error2
    }, this.updateParent);
  }

  onEditPassword2 = (event) => {
    const value = event.target.value;
    let error2 = null;

    if (value !== this.state.password1) {
      error2 = ERRORS.noMatchPassword;
    }

    this.setState({
      password2: value,
      password2Error: error2,
      isValidPass: !error2
    }, this.updateParent);
  }

  newError = (error) => {
    const { store } = this.context;

    store.dispatch({ type: 'newError', error });
  }
}
