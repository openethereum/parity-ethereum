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

import { Form, Input } from '~/ui';
import PasswordStrength from '~/ui/Form/PasswordStrength';

import ChangeVault from '../ChangeVault';
import styles from '../createAccount.css';

@observer
export default class RawKey extends Component {
  static contextTypes = {
    api: PropTypes.object.isRequired
  }

  static propTypes = {
    createStore: PropTypes.object.isRequired,
    vaultStore: PropTypes.object
  }

  render () {
    const { name, nameError, password, passwordRepeat, passwordRepeatError, passwordHint, rawKey, rawKeyError } = this.props.createStore;

    return (
      <Form>
        <Input
          autoFocus
          error={ rawKeyError }
          hint={
            <FormattedMessage
              id='createAccount.rawKey.private.hint'
              defaultMessage='the raw hex encoded private key'
            />
          }
          label={
            <FormattedMessage
              id='createAccount.rawKey.private.label'
              defaultMessage='private key'
            />
          }
          onChange={ this.onEditKey }
          value={ rawKey }
        />
        <Input
          error={ nameError }
          hint={
            <FormattedMessage
              id='createAccount.rawKey.name.hint'
              defaultMessage='a descriptive name for the account'
            />
          }
          label={
            <FormattedMessage
              id='createAccount.rawKey.name.label'
              defaultMessage='account name'
            />
          }
          onChange={ this.onEditName }
          value={ name }
        />
        <Input
          hint={
            <FormattedMessage
              id='createAccount.rawKey.hint.hint'
              defaultMessage='(optional) a hint to help with remembering the password'
            />
          }
          label={
            <FormattedMessage
              id='createAccount.rawKey.hint.label'
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
                  id='createAccount.rawKey.password.hint'
                  defaultMessage='a strong, unique password'
                />
              }
              label={
                <FormattedMessage
                  id='createAccount.rawKey.password.label'
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
                  id='createAccount.rawKey.password2.hint'
                  defaultMessage='verify your password'
                />
              }
              label={
                <FormattedMessage
                  id='createAccount.rawKey.password2.label'
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
      </Form>
    );
  }

  onEditName = (event, name) => {
    const { createStore } = this.props;

    createStore.setName(name);
  }

  onEditPasswordHint = (event, passwordHint) => {
    const { createStore } = this.props;

    createStore.setPasswordHint(passwordHint);
  }

  onEditPassword = (event, password) => {
    const { createStore } = this.props;

    createStore.setPassword(password);
  }

  onEditPasswordRepeat = (event, password) => {
    const { createStore } = this.props;

    createStore.setPasswordRepeat(password);
  }

  onEditKey = (event, rawKey) => {
    const { createStore } = this.props;

    createStore.setRawKey(rawKey);
  }
}
