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

import { Form, Input, Warning } from '~/ui';
import PasswordStrength from '~/ui/Form/PasswordStrength';

import ChangeVault from '../ChangeVault';
import styles from '../createAccount.css';

@observer
export default class RecoveryPhrase extends Component {
  static propTypes = {
    createStore: PropTypes.object.isRequired,
    vaultStore: PropTypes.object
  }

  render () {
    const { isWindowsPhrase, name, nameError, passPhraseError, password, passwordRepeat, passwordRepeatError, passwordHint, phrase } = this.props.createStore;

    return (
      <div className={ styles.details }>
        { this.renderWarning() }
        <Form>
          <Input
            autoFocus
            error={
              passPhraseError
              ? (
                <FormattedMessage
                  id='createAccount.recoveryPhrase.passPhrase.error'
                  defaultMessage='enter a recovery phrase'
                />
              )
              : null
            }
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
            createStore={ this.props.createStore }
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
      </div>
    );
  }

  renderWarning () {
    const { isTest, phrase } = this.props.createStore;

    if (!isTest && phrase.length === 0) {
      return (
        <Warning
          warning={
            <FormattedMessage
              id='createAccount.recoveryPhrase.warning.emptyPhrase'
              defaultMessage={ `The recovery phrase is empty.
                This account can be recovered by anyone.
              ` }
            />
          }
        />
      );
    }

    if (phrase.length === 0) {
      return (
        <Warning
          warning={
            <FormattedMessage
              id='createAccount.recoveryPhrase.warning.testnetEmptyPhrase'
              defaultMessage={ `The recovery phrase is empty.
                This account can be recovered by anyone.
                Proceed with caution.
              ` }
            />
          }
        />
      );
    }

    const words = phrase.split(' ');

    if (words.length < 11) {
      return (
        <Warning
          warning={
            <FormattedMessage
              id='createAccount.recoveryPhrase.warning.shortPhrase'
              defaultMessage={ `The recovery phrase is less than 11 words.
                This account has not been generated by Parity and might be insecure.
                Proceed with caution.
              ` }
            />
          }
        />
      );
    }

    return null;
  }

  onToggleWindowsPhrase = (event) => {
    const { createStore } = this.props;

    createStore.setWindowsPhrase(!createStore.isWindowsPhrase);
  }

  onEditPhrase = (event, phrase) => {
    const { createStore } = this.props;

    createStore.setPhrase(phrase);
  }

  onEditName = (event, name) => {
    const { createStore } = this.props;

    createStore.setName(name);
  }

  onEditPassword = (event, password) => {
    const { createStore } = this.props;

    createStore.setPassword(password);
  }

  onEditPasswordRepeat = (event, password) => {
    const { createStore } = this.props;

    createStore.setPasswordRepeat(password);
  }

  onEditPasswordHint = (event, passwordHint) => {
    const { createStore } = this.props;

    createStore.setPasswordHint(passwordHint);
  }
}
