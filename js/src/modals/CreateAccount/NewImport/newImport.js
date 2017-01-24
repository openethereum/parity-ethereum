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

import { FloatingActionButton } from 'material-ui';
import { observer } from 'mobx-react';
import React, { Component, PropTypes } from 'react';
import ReactDOM from 'react-dom';
import { FormattedMessage } from 'react-intl';

import { Form, Input } from '~/ui';
import { AttachFileIcon } from '~/ui/Icons';

import styles from '../createAccount.css';

const STYLE_HIDDEN = { display: 'none' };

@observer
export default class NewImport extends Component {
  static propTypes = {
    store: PropTypes.object.isRequired
  }

  render () {
    const { name, nameError, password, passwordHint, walletFile, walletFileError } = this.props.store;

    return (
      <Form>
        <Input
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
        <div>
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
          <div className={ styles.upload }>
            <FloatingActionButton
              mini
              onTouchTap={ this.openFileDialog }
            >
              <AttachFileIcon />
            </FloatingActionButton>
            <input
              onChange={ this.onFileChange }
              ref='fileUpload'
              style={ STYLE_HIDDEN }
              type='file'
            />
          </div>
        </div>
      </Form>
    );
  }

  onFileChange = (event) => {
    const { store } = this.props;

    if (event.target.files.length) {
      const reader = new FileReader();

      reader.onload = (event) => store.setWalletJson(event.target.result);
      reader.readAsText(event.target.files[0]);
    }

    store.setWalletFile(event.target.value);
  }

  openFileDialog = () => {
    ReactDOM.findDOMNode(this.refs.fileUpload).click();
  }

  onEditName = (event, name) => {
    const { store } = this.props;

    store.setName(name);
  }

  onEditPassword = (event, password) => {
    const { store } = this.props;

    store.setPassword(password);
  }

  onEditPasswordHint = (event, passwordHint) => {
    const { store } = this.props;

    store.setPasswordHint(passwordHint);
  }
}
