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
import ContentClear from 'material-ui/svg-icons/content/clear';
import CheckIcon from 'material-ui/svg-icons/navigation/check';
import SendIcon from 'material-ui/svg-icons/content/send';

import { Tabs, Tab } from 'material-ui/Tabs';
import Paper from 'material-ui/Paper';

import Form, { Input } from '../../ui/Form';
import { Button, Modal, IdentityName, IdentityIcon } from '../../ui';

import styles from './passwordManager.css';

const TEST_ACTION = 'TEST_ACTION';
const CHANGE_ACTION = 'CHANGE_ACTION';

export default class PasswordManager extends Component {
  static contextTypes = {
    api: PropTypes.object.isRequired
  }

  static propTypes = {
    account: PropTypes.object.isRequired,
    onClose: PropTypes.func
  }

  state = {
    action: TEST_ACTION,
    waiting: false,
    showMessage: false,
    message: { value: '', success: true },
    currentPass: '',
    newPass: '',
    repeatNewPass: '',
    repeatValid: true,
    passwordHint: this.props.account.meta && this.props.account.meta.passwordHint || ''
  }

  render () {
    return (
      <Modal
        actions={ this.renderDialogActions() }
        title='Password Manager'
        visible>
        { this.renderAccount() }
        { this.renderPage() }
        { this.renderMessage() }
      </Modal>
    );
  }

  renderMessage () {
    const { message, showMessage } = this.state;

    const style = message.success
      ? {
        backgroundColor: 'rgba(174, 213, 129, 0.75)'
      }
      : {
        backgroundColor: 'rgba(229, 115, 115, 0.75)'
      };

    const classes = [ styles.message ];

    if (!showMessage) {
      classes.push(styles.hideMessage);
    }

    return (
      <Paper
        zDepth={ 1 }
        style={ style }
        className={ classes.join(' ') }>
        { message.value }
      </Paper>
    );
  }

  renderAccount () {
    const { account } = this.props;
    const { address, meta } = account;

    const passwordHint = meta && meta.passwordHint
      ? (
      <span className={ styles.passwordHint }>
        <span className={ styles.hintLabel }>Hint </span>
        { meta.passwordHint }
      </span>
      )
      : null;

    return (
      <div className={ styles.accountContainer }>
        <IdentityIcon
          address={ address }
        />
        <div className={ styles.accountInfos }>
          <IdentityName
            className={ styles.accountName }
            address={ address }
            unknown
          />
          <span className={ styles.accountAddress }>
            { address }
          </span>
          { passwordHint }
        </div>
      </div>
    );
  }

  renderPage () {
    const { account } = this.props;
    const { waiting, repeatValid } = this.state;
    const disabled = !!waiting;

    const repeatError = repeatValid
      ? null
      : 'the two passwords differ';

    const { meta } = account;
    const passwordHint = meta && meta.passwordHint || '';

    return (
      <Tabs
        inkBarStyle={ {
          backgroundColor: 'rgba(255, 255, 255, 0.55)'
        } }
        tabItemContainerStyle={ {
          backgroundColor: 'rgba(255, 255, 255, 0.05)'
        } }
      >
        <Tab
          onActive={ this.handleTestActive }
          label='Test Password'
        >
          <Form
            className={ styles.form }
          >
            <div>
              <Input
                label='password'
                hint='your current password for this account'
                type='password'
                submitOnBlur={ false }
                disabled={ disabled }
                onSubmit={ this.handleTestPassword }
                onChange={ this.onEditCurrent } />
            </div>
          </Form>
        </Tab>
        <Tab
          onActive={ this.handleChangeActive }
          label='Change Password'
        >
          <Form
            className={ styles.form }
          >
            <div>
              <Input
                label='current password'
                hint='your current password for this account'
                type='password'
                submitOnBlur={ false }
                disabled={ disabled }
                onSubmit={ this.handleChangePassword }
                onChange={ this.onEditCurrent } />

              <Input
                label='new password'
                hint='the new password for this account'
                type='password'
                submitOnBlur={ false }
                disabled={ disabled }
                onSubmit={ this.handleChangePassword }
                onChange={ this.onEditNew } />
              <Input
                label='repeat new password'
                hint='repeat the new password for this account'
                type='password'
                submitOnBlur={ false }
                error={ repeatError }
                disabled={ disabled }
                onSubmit={ this.handleChangePassword }
                onChange={ this.onEditRepeatNew } />

              <Input
                label='new password hint'
                hint='hint for the new password'
                submitOnBlur={ false }
                value={ passwordHint }
                disabled={ disabled }
                onSubmit={ this.handleChangePassword }
                onChange={ this.onEditHint } />
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
        icon={ <ContentClear /> }
        label='Cancel'
        onClick={ onClose } />
    );

    if (waiting) {
      const waitingBtn = (
        <Button
          disabled
          label='Wait...' />
      );

      return [ cancelBtn, waitingBtn ];
    }

    if (action === TEST_ACTION) {
      const testBtn = (
        <Button
          icon={ <CheckIcon /> }
          label='Test'
          onClick={ this.handleTestPassword } />
      );

      return [ cancelBtn, testBtn ];
    }

    const changeBtn = (
      <Button
        disabled={ !repeatValid }
        icon={ <SendIcon /> }
        label='Change'
        onClick={ this.handleChangePassword } />
    );

    return [ cancelBtn, changeBtn ];
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
    const repeatValid = value === this.state.newPass;

    this.setState({
      repeatNewPass: value,
      showMessage: false,
      repeatValid
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

  handleTestPassword = () => {
    const { account } = this.props;
    const { currentPass } = this.state;

    this.setState({ waiting: true, showMessage: false });

    this.context
      .api.personal
      .testPassword(account.address, currentPass)
      .then(correct => {
        const message = correct
          ? { value: 'This password is correct', success: true }
          : { value: 'This password is not correct', success: false };

        this.setState({ waiting: false, message, showMessage: true });
      })
      .catch(e => {
        console.error('passwordManager::handleTestPassword', e);
        this.setState({ waiting: false });
      });
  }

  handleChangePassword = () => {
    const { account } = this.props;
    const { currentPass, newPass, repeatNewPass, passwordHint } = this.state;

    if (repeatNewPass !== newPass) {
      return;
    }

    this.setState({ waiting: true, showMessage: false });

    this.context
      .api.personal
      .testPassword(account.address, currentPass)
      .then(correct => {
        if (!correct) {
          const message = {
            value: 'This provided current password is not correct',
            success: false
          };

          this.setState({ waiting: false, message, showMessage: true });

          return false;
        }

        const meta = Object.assign({}, account.meta, {
          passwordHint
        });

        return Promise.all([
          this.context
            .api.personal
            .setAccountMeta(account.address, meta),

          this.context
            .api.personal
            .changePassword(account.address, currentPass, newPass)
        ])
          .then(() => {
            const message = {
              value: 'Your password has been successfully changed',
              success: true
            };

            this.setState({ waiting: false, message, showMessage: true });
          });
      })
      .catch(e => {
        console.error('passwordManager::handleChangePassword', e);
        this.setState({ waiting: false });
      });
  }
}
