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
import ReactDOM from 'react-dom';
import { FloatingActionButton } from 'material-ui';
import EditorAttachFile from 'material-ui/svg-icons/editor/attach-file';

import { Form, Input } from '~/ui';

import ERRORS from '../errors';

import styles from '../createAccount.css';

const FAKEPATH = 'C:\\fakepath\\';
const STYLE_HIDDEN = { display: 'none' };

export default class NewImport extends Component {
  static propTypes = {
    onChange: PropTypes.func.isRequired
  }

  state = {
    accountName: '',
    accountNameError: ERRORS.noName,
    isValidFile: false,
    isValidPass: true,
    isValidName: false,
    password: '',
    passwordError: null,
    passwordHint: '',
    walletFile: '',
    walletFileError: ERRORS.noFile,
    walletJson: ''
  }

  componentWillMount () {
    this.props.onChange(false, {});
  }

  render () {
    return (
      <Form>
        <Input
          label='account name'
          hint='a descriptive name for the account'
          error={ this.state.accountNameError }
          value={ this.state.accountName }
          onChange={ this.onEditAccountName }
        />
        <Input
          label='password hint'
          hint='(optional) a hint to help with remembering the password'
          value={ this.state.passwordHint }
          onChange={ this.onEditpasswordHint }
        />
        <div className={ styles.passwords }>
          <div className={ styles.password }>
            <Input
              label='password'
              hint='the password to unlock the wallet'
              type='password'
              error={ this.state.passwordError }
              value={ this.state.password }
              onChange={ this.onEditPassword }
            />
          </div>
        </div>
        <div>
          <Input
            disabled
            label='wallet file'
            hint='the wallet file for import'
            error={ this.state.walletFileError }
            value={ this.state.walletFile }
          />
          <div className={ styles.upload }>
            <FloatingActionButton
              mini
              onTouchTap={ this.openFileDialog }
            >
              <EditorAttachFile />
            </FloatingActionButton>
            <input
              ref='fileUpload'
              type='file'
              style={ STYLE_HIDDEN }
              onChange={ this.onFileChange }
            />
          </div>
        </div>
      </Form>
    );
  }

  onFileChange = (event) => {
    const el = event.target;
    const error = ERRORS.noFile;

    if (el.files.length) {
      const reader = new FileReader();

      reader.onload = (event) => {
        this.setState({
          walletJson: event.target.result,
          walletFileError: null,
          isValidFile: true
        }, this.updateParent);
      };
      reader.readAsText(el.files[0]);
    }

    this.setState({
      walletFile: el.value.replace(FAKEPATH, ''),
      walletFileError: error,
      isValidFile: false
    }, this.updateParent);
  }

  openFileDialog = () => {
    ReactDOM.findDOMNode(this.refs.fileUpload).click();
  }

  updateParent = () => {
    const valid = this.state.isValidName && this.state.isValidPass && this.state.isValidFile;

    this.props.onChange(valid, {
      name: this.state.accountName,
      passwordHint: this.state.passwordHint,
      password: this.state.password,
      phrase: null,
      json: this.state.walletJson
    });
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

  onEditPassword = (event) => {
    const password = event.target.value;

    this.setState({
      password,
      passwordError: null,
      isValidPass: true
    }, this.updateParent);
  }
}
