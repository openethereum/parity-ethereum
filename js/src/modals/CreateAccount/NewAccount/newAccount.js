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
import IconButton from 'material-ui/IconButton';
import { RadioButton, RadioButtonGroup } from 'material-ui/RadioButton';

import { newError } from '~/redux/actions';
import { Form, Input, IdentityIcon } from '~/ui';
import { RefreshIcon } from '~/ui/Icons';

import ERRORS from '../errors';

import styles from '../createAccount.css';

@observer
export default class CreateAccount extends Component {
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
    accounts: null,
    isValidName: false,
    isValidPass: false,
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
    const { passwordHint } = this.props.store;
    const { accountName, accountNameError, password1, password1Error, password2, password2Error } = this.state;

    return (
      <Form>
        <Input
          error={ accountNameError }
          hint={
            <FormattedMessage
              id='createAccount.newAccount.name.hint'
              defaultMessage='a descriptive name for the account' />
          }
          label={
            <FormattedMessage
              id='createAccount.newAccount.name.label'
              defaultMessage='account name' />
          }
          onChange={ this.onEditAccountName }
          value={ accountName } />
        <Input
          hint={
            <FormattedMessage
              id='createAccount.newAccount.hint.hint'
              defaultMessage='(optional) a hint to help with remembering the password' />
          }
          label={
            <FormattedMessage
              id='createAccount.newAccount.hint.label'
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
                  id='createAccount.newAccount.password.hint'
                  defaultMessage='a strong, unique password' />
              }
              label={
                <FormattedMessage
                  id='createAccount.newAccount.password.label'
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
                  id='createAccount.newAccount.password2.hint'
                  defaultMessage='verify your password' />
              }
              label={
                <FormattedMessage
                  id='createAccount.newAccount.password2.label'
                  defaultMessage='password (repeat)' />
              }
              onChange={ this.onEditPassword2 }
              type='password'
              value={ password2 } />
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

    const buttons = Object
      .keys(accounts)
      .map((address) => {
        return (
          <RadioButton
            className={ styles.button }
            key={ address }
            value={ address } />
        );
      });

    return (
      <RadioButtonGroup
        className={ styles.selector }
        name='identitySelector'
        onChange={ this.onChangeIdentity }
        valueSelected={ selectedAddress }>
        { buttons }
      </RadioButtonGroup>
    );
  }

  renderIdentities () {
    const { accounts } = this.state;

    if (!accounts) {
      return null;
    }

    const identities = Object
      .keys(accounts)
      .map((address) => {
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
            <RefreshIcon color='rgb(0, 151, 167)' />
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
        newError(error);
      });
  }

  updateParent = () => {
    const { isValidName, isValidPass, accounts, accountName, password1, selectedAddress } = this.state;
    const isValid = isValidName && isValidPass;

    this.props.onChange(isValid, {
      address: selectedAddress,
      name: accountName,
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
    const { store } = this.props;

    store.setPasswordHint(passwordHint);
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
