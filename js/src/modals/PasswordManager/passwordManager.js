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

import Paper from 'material-ui/Paper';
import { Tabs, Tab } from 'material-ui/Tabs';
import React, { Component, PropTypes } from 'react';
import { FormattedMessage } from 'react-intl';

import { Button, Modal, IdentityName, IdentityIcon } from '~/ui';
import Form, { Input } from '~/ui/Form';
import { CancelIcon, CheckIcon, SendIcon } from '~/ui/Icons';

import Store from './store';
import styles from './passwordManager.css';

const TEST_ACTION = 'TEST_ACTION';
const CHANGE_ACTION = 'CHANGE_ACTION';

const MSG_SUCCESS_STYLE = {
  backgroundColor: 'rgba(174, 213, 129, 0.75)'
};
const MSG_FAILURE_STYLE = {
  backgroundColor: 'rgba(229, 115, 115, 0.75)'
};
const TABS_INKBAR_STYLE = {
  backgroundColor: 'rgba(255, 255, 255, 0.55)'
};
const TABS_ITEM_STYLE = {
  backgroundColor: 'rgba(255, 255, 255, 0.05)'
};

export default class PasswordManager extends Component {
  static contextTypes = {
    api: PropTypes.object.isRequired
  }

  static propTypes = {
    account: PropTypes.object.isRequired,
    onClose: PropTypes.func
  }

  store = new Store(this.context.api, this.props.account);

  state = {
    action: TEST_ACTION,
    waiting: false,
    showMessage: false,
    message: { value: '', success: true },
    repeatNewPassValid: true
  }

  render () {
    return (
      <Modal
        actions={ this.renderDialogActions() }
        title={
          <FormattedMessage
            id='passwordChange.title'
            defaultMessage='Password Manager' />
        }
        visible>
        { this.renderAccount() }
        { this.renderPage() }
        { this.renderMessage() }
      </Modal>
    );
  }

  renderMessage () {
    const { message, showMessage } = this.state;
    const classes = [styles.message];

    if (!showMessage) {
      classes.push(styles.hideMessage);
    }

    return (
      <Paper
        className={ classes.join(' ') }
        style={
          message.success
            ? MSG_SUCCESS_STYLE
            : MSG_FAILURE_STYLE
        }
        zDepth={ 1 }>
        { message.value }
      </Paper>
    );
  }

  renderAccount () {
    const { address, passwordHint } = this.store;

    return (
      <div className={ styles.accountContainer }>
        <IdentityIcon address={ address } />
        <div className={ styles.accountInfos }>
          <IdentityName
            address={ address }
            className={ styles.accountName }
            unknown />
          <span className={ styles.accountAddress }>
            { address }
          </span>
          <span className={ styles.passwordHint }>
            <span className={ styles.hintLabel }>Hint </span>
            { passwordHint || '-' }
          </span>
        </div>
      </div>
    );
  }

  renderPage () {
    const { isRepeatValid, passwordHint } = this.store;
    const { waiting } = this.state;
    const disabled = !!waiting;

    return (
      <Tabs
        inkBarStyle={ TABS_INKBAR_STYLE }
        tabItemContainerStyle={ TABS_ITEM_STYLE }>
        <Tab
          label={
            <FormattedMessage
              id='passwordChange.tabTest.label'
              defaultMessage='Test Password' />
          }
          onActive={ this.handleTestActive }>
          <Form className={ styles.form }>
            <div>
              <Input
                disabled={ disabled }
                hint={
                  <FormattedMessage
                    id='passwordChange.testPassword.hint'
                    defaultMessage='your account password' />
                }
                label={
                  <FormattedMessage
                    id='passwordChange.testPassword.label'
                    defaultMessage='password' />
                }
                onChange={ this.onEditCurrent }
                onSubmit={ this.handleTestPassword }
                submitOnBlur={ false }
                type='password' />
            </div>
          </Form>
        </Tab>
        <Tab
          label={
            <FormattedMessage
              id='passwordChange.tabChange.label'
              defaultMessage='Change Password' />
          }
          onActive={ this.handleChangeActive }>
          <Form className={ styles.form }>
            <div>
              <Input
                disabled={ disabled }
                hint={
                  <FormattedMessage
                    id='passwordChange.currentPassword.hint'
                    defaultMessage='your current password for this account' />
                }
                label={
                  <FormattedMessage
                    id='passwordChange.currentPassword.label'
                    defaultMessage='current password' />
                }
                onChange={ this.onEditCurrent }
                onSubmit={ this.handleChangePassword }
                submitOnBlur={ false }
                type='password' />
              <Input
                disabled={ disabled }
                hint={
                  <FormattedMessage
                    id='passwordChange.passwordHint.hint'
                    defaultMessage='hint for the new password' />
                }
                label={
                  <FormattedMessage
                    id='passwordChange.passwordHint.label'
                    defaultMessage='(optional) new password hint' />
                }
                onChange={ this.onEditHint }
                onSubmit={ this.handleChangePassword }
                submitOnBlur={ false }
                value={ passwordHint } />
              <div className={ styles.passwords }>
                <div className={ styles.password }>
                  <Input
                    disabled={ disabled }
                    hint={
                      <FormattedMessage
                        id='passwordChange.newPassword.hint'
                        defaultMessage='the new password for this account' />
                    }
                    label={
                      <FormattedMessage
                        id='passwordChange.newPassword.label'
                        defaultMessage='new password' />
                    }
                    onChange={ this.onEditNew }
                    onSubmit={ this.handleChangePassword }
                    submitOnBlur={ false }
                    type='password' />
                </div>
                <div className={ styles.password }>
                  <Input
                    disabled={ disabled }
                    error={
                      isRepeatValid
                        ? null
                        : <FormattedMessage
                          id='passwordChange.repeatPassword.error'
                          defaultMessage='the supplied passwords do not match' />
                    }
                    hint={
                      <FormattedMessage
                        id='passwordChange.repeatPassword.hint'
                        defaultMessage='repeat the new password for this account' />
                    }
                    label={
                      <FormattedMessage
                        id='passwordChange.repeatPassword.label'
                        defaultMessage='repeat new password' />
                    }
                    onChange={ this.onEditRepeatNew }
                    onSubmit={ this.handleChangePassword }
                    submitOnBlur={ false }
                    type='password' />
                </div>
              </div>
            </div>
          </Form>
        </Tab>
      </Tabs>
    );
  }

  renderDialogActions () {
    const { onClose } = this.props;
    const { action, waiting, repeatValid } = this.state;

    const cancelBtn = (
      <Button
        icon={ <CancelIcon /> }
        key='cancel'
        label={
          <FormattedMessage
            id='passwordChange.button.cancel'
            defaultMessage='Cancel' />
        }
        onClick={ onClose } />
    );

    if (waiting) {
      return [
        cancelBtn,
        <Button
          disabled
          key='wait'
          label={
            <FormattedMessage
              id='passwordChange.button.wait'
              defaultMessage='Wait...' />
          } />
      ];
    }

    if (action === TEST_ACTION) {
      return [
        cancelBtn,
        <Button
          icon={ <CheckIcon /> }
          key='test'
          label={
            <FormattedMessage
              id='passwordChange.button.test'
              defaultMessage='Test' />
          }
          onClick={ this.handleTestPassword } />
      ];
    }

    return [
      cancelBtn,
      <Button
        disabled={ !repeatValid }
        icon={ <SendIcon /> }
        key='change'
        label={
          <FormattedMessage
            id='passwordChange.button.change'
            defaultMessage='Change' />
        }
        onClick={ this.handleChangePassword } />
    ];
  }

  onEditCurrent = (event, value) => {
    this.setState({
      currentPass: value,
      showMessage: false
    });
  }

  onEditNew = (event, value) => {
    const repeatValid = value === this.state.repeatNewPass;

    this.setState({
      newPass: value,
      showMessage: false,
      repeatValid
    });
  }

  onEditRepeatNew = (event, value) => {
    this.setState({
      repeatNewPass: value,
      showMessage: false
    });
  }

  onEditHint = (event, value) => {
    this.setState({
      passwordHint: value,
      showMessage: false
    });
  }

  handleTestActive = () => {
    this.setState({
      action: TEST_ACTION,
      showMessage: false
    });
  }

  handleChangeActive = () => {
    this.setState({
      action: CHANGE_ACTION,
      showMessage: false
    });
  }
}
