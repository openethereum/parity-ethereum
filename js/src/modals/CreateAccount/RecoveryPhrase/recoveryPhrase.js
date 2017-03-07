// Copyright 2015-2017 Parity Technologies (UK) Ltd.
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
import { Checkbox } from 'material-ui';

import { Form, Input } from '~/ui';
import PasswordStrength from '~/ui/Form/PasswordStrength';

import ChangeVault from '../ChangeVault';
import styles from '../createAccount.css';

@observer
export default class RecoveryPhrase extends Component {
  static propTypes = {
    store: PropTypes.object.isRequired,
    vaultStore: PropTypes.object
  }

  render () {
    const { isWindowsPhrase, name, nameError, password, passwordRepeat, passwordRepeatError, passwordHint, phrase } = this.props.store;

    return (
      <Form>
        <Input
          autoFocus
          hint={
            <FormattedMessage
              id='createAccount.recoveryPhrase.phrase.hint'
              defaultMessage='the account recovery phrase'
            />
          }
          label={
            <FormattedMessage
              id='createAccount.recoveryPhrase.phrase.label'
              defaultMessage='account recovery phrase'
            />
          }
          onChange={ this.onEditPhrase }
          value={ phrase }
        />
        <Input
          error={ nameError }
          hint={
            <FormattedMessage
              id='createAccount.recoveryPhrase.name.hint'
              defaultMessage='a descriptive name for the account'
            />
          }
          label={
            <FormattedMessage
              id='createAccount.recoveryPhrase.name.label'
              defaultMessage='account name'
            />
          }
          onChange={ this.onEditName }
          value={ name }
        />
        <Input
          hint={
            <FormattedMessage
              id='createAccount.recoveryPhrase.hint.hint'
              defaultMessage='(optional) a hint to help with remembering the password'
            />
          }
          label={
            <FormattedMessage
              id='createAccount.recoveryPhrase.hint.label'
              defaultMessage='password hint'
            />
          }
          onChange={ this.onEditPasswordHint }
          value={ passwordHint }
        />
        <div className={ styles.passwords }>
          <div className={ styles.password }>
            <Input
              hint={
                <FormattedMessage
                  id='createAccount.recoveryPhrase.password.hint'
                  defaultMessage='a strong, unique password'
                />
              }
              label={
                <FormattedMessage
                  id='createAccount.recoveryPhrase.password.label'
                  defaultMessage='password'
                />
              }
              onChange={ this.onEditPassword }
              type='password'
              value={ password }
            />
          </div>
          <div className={ styles.password }>
            <Input
              error={ passwordRepeatError }
              hint={
                <FormattedMessage
                  id='createAccount.recoveryPhrase.password2.hint'
                  defaultMessage='verify your password'
                />
              }
              label={
                <FormattedMessage
                  id='createAccount.recoveryPhrase.password2.label'
                  defaultMessage='password (repeat)'
                />
              }
              onChange={ this.onEditPasswordRepeat }
              type='password'
              value={ passwordRepeat }
            />
          </div>
        </div>
        <PasswordStrength input={ password } />
        <ChangeVault
          store={ this.props.store }
          vaultStore={ this.props.vaultStore }
        />
        <Checkbox
          checked={ isWindowsPhrase }
          className={ styles.checkbox }
          label={
            <FormattedMessage
              id='createAccount.recoveryPhrase.windowsKey.label'
              defaultMessage='Key was created with Parity <1.4.5 on Windows'
            />
          }
          onCheck={ this.onToggleWindowsPhrase }
        />
      </Form>
    );
  }

  onToggleWindowsPhrase = (event) => {
    const { store } = this.props;

    store.setWindowsPhrase(!store.isWindowsPhrase);
  }

  onEditPhrase = (event, phrase) => {
    const { store } = this.props;

    store.setPhrase(phrase);
  }

  onEditName = (event, name) => {
    const { store } = this.props;

    store.setName(name);
  }

  onEditPassword = (event, password) => {
    const { store } = this.props;

    store.setPassword(password);
  }

  onEditPasswordRepeat = (event, password) => {
    const { store } = this.props;

    store.setPasswordRepeat(password);
  }

  onEditPasswordHint = (event, passwordHint) => {
    const { store } = this.props;

    store.setPasswordHint(passwordHint);
  }
}
