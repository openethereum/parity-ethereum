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

import Paper from 'material-ui/Paper';
import { Tabs, Tab } from 'material-ui/Tabs';
import { observer } from 'mobx-react';
import React, { Component, PropTypes } from 'react';
import { FormattedMessage } from 'react-intl';
import { connect } from 'react-redux';
import { bindActionCreators } from 'redux';

import { newError, openSnackbar } from '~/redux/actions';
import { Button, IdentityName, IdentityIcon, Portal } from '~/ui';
import PasswordStrength from '~/ui/Form/PasswordStrength';
import Form, { Input } from '~/ui/Form';
import { CancelIcon, CheckIcon, SendIcon } from '~/ui/Icons';

import Store, { CHANGE_ACTION, TEST_ACTION } from './store';
import styles from './passwordManager.css';

const MSG_SUCCESS_STYLE = {
  backgroundColor: 'rgba(174, 213, 129, 0.75)'
};
const MSG_FAILURE_STYLE = {
  backgroundColor: 'rgba(229, 115, 115, 0.75)'
};
const TABS_INKBAR_STYLE = {
  backgroundColor: 'rgb(0, 151, 167)' // 'rgba(255, 255, 255, 0.55)'
};
const TABS_ITEM_STYLE = {
  backgroundColor: 'rgba(255, 255, 255, 0.05)'
};

@observer
class PasswordManager extends Component {
  static contextTypes = {
    api: PropTypes.object.isRequired
  }

  static propTypes = {
    account: PropTypes.object.isRequired,
    openSnackbar: PropTypes.func.isRequired,
    newError: PropTypes.func.isRequired,
    onClose: PropTypes.func
  }

  store = new Store(this.context.api, this.props.account);

  render () {
    const { busy } = this.store;

    return (
      <Portal
        busy={ busy }
        buttons={ this.renderDialogActions() }
        onClose={ this.onClose }
        open
        title={
          <FormattedMessage
            id='passwordChange.title'
            defaultMessage='Password Manager'
          />
        }
      >
        { this.renderAccount() }
        { this.renderPage() }
        { this.renderMessage() }
      </Portal>
    );
  }

  renderMessage () {
    const { infoMessage } = this.store;

    if (!infoMessage) {
      return null;
    }

    return (
      <Paper
        className={ `${styles.message}` }
        style={
          infoMessage.success
            ? MSG_SUCCESS_STYLE
            : MSG_FAILURE_STYLE
        }
        zDepth={ 1 }
      >
        { infoMessage.value }
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
            unknown
          />
          <span className={ styles.accountAddress }>
            { address }
          </span>
          <span className={ styles.passwordHint }>
            <span className={ styles.hintLabel }>
              <FormattedMessage
                id='passwordChange.passwordHint.display'
                defaultMessage='Hint {hint}'
                values={ {
                  hint: passwordHint || '-'
                } }
              />
            </span>
          </span>
        </div>
      </div>
    );
  }

  renderPage () {
    return (
      <Tabs
        inkBarStyle={ TABS_INKBAR_STYLE }
        tabItemContainerStyle={ TABS_ITEM_STYLE }
      >
        <Tab
          label={
            <FormattedMessage
              id='passwordChange.tabTest.label'
              defaultMessage='Test Password'
            />
          }
          onActive={ this.onActivateTestTab }
        >
          { this.renderTabTest() }
        </Tab>
        <Tab
          label={
            <FormattedMessage
              id='passwordChange.tabChange.label'
              defaultMessage='Change Password'
            />
          }
          onActive={ this.onActivateChangeTab }
        >
          { this.renderTabChange() }
        </Tab>
      </Tabs>
    );
  }

  renderTabTest () {
    const { actionTab, busy } = this.store;

    if (actionTab !== TEST_ACTION) {
      return null;
    }

    return (
      <Form className={ styles.form }>
        <div>
          <Input
            autoFocus
            disabled={ busy }
            hint={
              <FormattedMessage
                id='passwordChange.testPassword.hint'
                defaultMessage='your account password'
              />
            }
            label={
              <FormattedMessage
                id='passwordChange.testPassword.label'
                defaultMessage='password'
              />
            }
            onChange={ this.onEditTestPassword }
            onSubmit={ this.testPassword }
            submitOnBlur={ false }
            type='password'
          />
        </div>
      </Form>
    );
  }

  renderTabChange () {
    const { actionTab, busy, isRepeatValid, newPassword, passwordHint } = this.store;

    if (actionTab !== CHANGE_ACTION) {
      return null;
    }

    return (
      <Form className={ styles.form }>
        <div>
          <Input
            autoFocus
            disabled={ busy }
            hint={
              <FormattedMessage
                id='passwordChange.currentPassword.hint'
                defaultMessage='your current password for this account'
              />
            }
            label={
              <FormattedMessage
                id='passwordChange.currentPassword.label'
                defaultMessage='current password'
              />
            }
            onChange={ this.onEditCurrentPassword }
            type='password'
          />
          <Input
            disabled={ busy }
            hint={
              <FormattedMessage
                id='passwordChange.passwordHint.hint'
                defaultMessage='hint for the new password'
              />
            }
            label={
              <FormattedMessage
                id='passwordChange.passwordHint.label'
                defaultMessage='(optional) new password hint'
              />
            }
            onChange={ this.onEditNewPasswordHint }
            value={ passwordHint }
          />
          <div className={ styles.passwords }>
            <div className={ styles.password }>
              <Input
                disabled={ busy }
                hint={
                  <FormattedMessage
                    id='passwordChange.newPassword.hint'
                    defaultMessage='the new password for this account'
                  />
                }
                label={
                  <FormattedMessage
                    id='passwordChange.newPassword.label'
                    defaultMessage='new password'
                  />
                }
                onChange={ this.onEditNewPassword }
                onSubmit={ this.changePassword }
                submitOnBlur={ false }
                type='password'
              />
            </div>
            <div className={ styles.password }>
              <Input
                disabled={ busy }
                error={
                  isRepeatValid
                    ? null
                    : <FormattedMessage
                      id='passwordChange.repeatPassword.error'
                      defaultMessage='the supplied passwords do not match'
                      />
                }
                hint={
                  <FormattedMessage
                    id='passwordChange.repeatPassword.hint'
                    defaultMessage='repeat the new password for this account'
                  />
                }
                label={
                  <FormattedMessage
                    id='passwordChange.repeatPassword.label'
                    defaultMessage='repeat new password'
                  />
                }
                onChange={ this.onEditNewPasswordRepeat }
                onSubmit={ this.changePassword }
                submitOnBlur={ false }
                type='password'
              />
            </div>
          </div>

          <PasswordStrength input={ newPassword } />
        </div>
      </Form>
    );
  }

  renderDialogActions () {
    const { actionTab, busy, isRepeatValid } = this.store;

    const cancelBtn = (
      <Button
        icon={ <CancelIcon /> }
        key='cancel'
        label={
          <FormattedMessage
            id='passwordChange.button.cancel'
            defaultMessage='Cancel'
          />
        }
        onClick={ this.onClose }
      />
    );

    if (busy) {
      return [
        cancelBtn,
        <Button
          disabled
          key='wait'
          label={
            <FormattedMessage
              id='passwordChange.button.wait'
              defaultMessage='Wait...'
            />
          }
        />
      ];
    }

    if (actionTab === TEST_ACTION) {
      return [
        cancelBtn,
        <Button
          icon={ <CheckIcon /> }
          key='test'
          label={
            <FormattedMessage
              id='passwordChange.button.test'
              defaultMessage='Test'
            />
          }
          onClick={ this.testPassword }
        />
      ];
    }

    return [
      cancelBtn,
      <Button
        disabled={ !isRepeatValid }
        icon={ <SendIcon /> }
        key='change'
        label={
          <FormattedMessage
            id='passwordChange.button.change'
            defaultMessage='Change'
          />
        }
        onClick={ this.changePassword }
      />
    ];
  }

  onActivateChangeTab = () => {
    this.store.setActionTab(CHANGE_ACTION);
  }

  onActivateTestTab = () => {
    this.store.setActionTab(TEST_ACTION);
  }

  onEditCurrentPassword = (event, password) => {
    this.store.setPassword(password);
  }

  onEditNewPassword = (event, password) => {
    this.store.setNewPassword(password);
  }

  onEditNewPasswordHint = (event, passwordHint) => {
    this.store.setNewPasswordHint(passwordHint);
  }

  onEditNewPasswordRepeat = (event, password) => {
    this.store.setNewPasswordRepeat(password);
  }

  onEditTestPassword = (event, password) => {
    this.store.setValidatePassword(password);
  }

  onClose = () => {
    this.props.onClose();
  }

  changePassword = () => {
    return this.store
      .changePassword()
      .then((result) => {
        if (result) {
          this.props.openSnackbar(
            <div>
              <FormattedMessage
                id='passwordChange.success'
                defaultMessage='Your password has been successfully changed'
              />
            </div>
          );
          this.onClose();
        }
      })
      .catch((error) => {
        this.props.newError(error);
      });
  }

  testPassword = () => {
    return this.store
      .testPassword()
      .catch((error) => {
        this.props.newError(error);
      });
  }
}

function mapDispatchToProps (dispatch) {
  return bindActionCreators({
    openSnackbar,
    newError
  }, dispatch);
}

export default connect(
  null,
  mapDispatchToProps
)(PasswordManager);
