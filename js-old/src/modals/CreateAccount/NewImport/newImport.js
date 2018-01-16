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

import { Form, FileSelect, Input } from '~/ui';

import ChangeVault from '../ChangeVault';
import styles from '../createAccount.css';

@observer
export default class NewImport extends Component {
  static propTypes = {
    createStore: PropTypes.object.isRequired,
    vaultStore: PropTypes.object

  }

  render () {
    const { name, nameError, password, passwordHint } = this.props.createStore;

    return (
      <Form>
        <Input
          autoFocus
          error={ nameError }
          hint={
            <FormattedMessage
              id='createAccount.newImport.name.hint'
              defaultMessage='a descriptive name for the account'
            />
          }
          label={
            <FormattedMessage
              id='createAccount.newImport.name.label'
              defaultMessage='account name'
            />
          }
          onChange={ this.onEditName }
          value={ name }
        />
        <Input
          hint={
            <FormattedMessage
              id='createAccount.newImport.hint.hint'
              defaultMessage='(optional) a hint to help with remembering the password'
            />
          }
          label={
            <FormattedMessage
              id='createAccount.newImport.hint.label'
              defaultMessage='password hint'
            />
          }
          onChange={ this.onEditpasswordHint }
          value={ passwordHint }
        />
        <div className={ styles.passwords }>
          <div className={ styles.password }>
            <Input
              hint={
                <FormattedMessage
                  id='createAccount.newImport.password.hint'
                  defaultMessage='the password to unlock the wallet'
                />
              }
              label={
                <FormattedMessage
                  id='createAccount.newImport.password.label'
                  defaultMessage='password'
                />
              }
              type='password'
              onChange={ this.onEditPassword }
              value={ password }
            />
          </div>
        </div>
        <ChangeVault
          createStore={ this.props.createStore }
          vaultStore={ this.props.vaultStore }
        />
        { this.renderFileSelector() }
      </Form>
    );
  }

  renderFileSelector () {
    const { walletFile, walletFileError } = this.props.createStore;

    return walletFile
      ? (
        <Input
          disabled
          error={ walletFileError }
          hint={
            <FormattedMessage
              id='createAccount.newImport.file.hint'
              defaultMessage='the wallet file for import'
            />
          }
          label={
            <FormattedMessage
              id='createAccount.newImport.file.label'
              defaultMessage='wallet file'
            />
          }
          value={ walletFile }
        />
      )
      : (
        <FileSelect
          className={ styles.fileImport }
          error={ walletFileError }
          onSelect={ this.onFileSelect }
        />
      );
  }

  onFileSelect = (fileName, fileContent) => {
    const { createStore } = this.props;

    createStore.setWalletFile(fileName);
    createStore.setWalletJson(fileContent);
  }

  onEditName = (event, name) => {
    const { createStore } = this.props;

    createStore.setName(name);
  }

  onEditPassword = (event, password) => {
    const { createStore } = this.props;

    createStore.setPassword(password);
  }

  onEditPasswordHint = (event, passwordHint) => {
    const { createStore } = this.props;

    createStore.setPasswordHint(passwordHint);
  }
}
