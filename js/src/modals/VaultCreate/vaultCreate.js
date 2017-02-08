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
import { connect } from 'react-redux';
import { bindActionCreators } from 'redux';

import { newError } from '~/redux/actions';
import { Button, Input, Portal } from '~/ui';
import PasswordStrength from '~/ui/Form/PasswordStrength';
import { CheckIcon } from '~/ui/Icons';

import styles from './vaultCreate.css';

@observer
class VaultCreate extends Component {
  static propTypes = {
    newError: PropTypes.func.isRequired,
    vaultStore: PropTypes.object.isRequired
  }

  render () {
    const { createName, createNameError, createPassword, createPasswordHint, createPasswordRepeat, createPasswordRepeatError, isBusyCreate, isModalCreateOpen } = this.props.vaultStore;
    const hasError = !!createNameError || !!createPasswordRepeatError;

    if (!isModalCreateOpen) {
      return null;
    }

    return (
      <Portal
        buttons={
          <Button
            disabled={ hasError || isBusyCreate }
            icon={ <CheckIcon /> }
            label={
              <FormattedMessage
                id='vaults.create.button.create'
                defaultMessage='create vault'
              />
            }
            onClick={ this.onClickCreate }
          />
        }
        onClose={ this.onClose }
        open
        title={
          <FormattedMessage
            id='vaults.create.title'
            defaultMessage='Create a new vault'
          />
        }
      >
        <div className={ styles.body }>
          <Input
            error={ createNameError }
            hint={
              <FormattedMessage
                id='vaults.create.name.hint'
                defaultMessage='a descriptive name for the vault'
              />
            }
            label={
              <FormattedMessage
                id='vaults.create.name.label'
                defaultMessage='vault name'
              />
            }
            onChange={ this.onEditName }
            value={ createName }
          />
          <Input
            hint={
              <FormattedMessage
                id='vaults.create.hint.hint'
                defaultMessage='(optional) a hint to help with remembering the password'
              />
            }
            label={
              <FormattedMessage
                id='vaults.create.hint.label'
                defaultMessage='password hint'
              />
            }
            onChange={ this.onEditPasswordHint }
            value={ createPasswordHint }
          />
          <div className={ styles.passwords }>
            <div className={ styles.password }>
              <Input
                hint={
                  <FormattedMessage
                    id='vaults.create.password.hint'
                    defaultMessage='a strong, unique password'
                  />
                }
                label={
                  <FormattedMessage
                    id='vaults.create.password.label'
                    defaultMessage='password'
                  />
                }
                onChange={ this.onEditPassword }
                type='password'
                value={ createPassword }
              />
            </div>
            <div className={ styles.password }>
              <Input
                error={ createPasswordRepeatError }
                hint={
                  <FormattedMessage
                    id='vaults.create.password2.hint'
                    defaultMessage='verify your password'
                  />
                }
                label={
                  <FormattedMessage
                    id='vaults.create.password2.label'
                    defaultMessage='password (repeat)'
                  />
                }
                onChange={ this.onEditPasswordRepeat }
                type='password'
                value={ createPasswordRepeat }
              />
            </div>
          </div>
          <PasswordStrength input={ createPassword } />
        </div>
      </Portal>
    );
  }

  onEditName = (event, name) => {
    this.props.vaultStore.setCreateName(name);
  }

  onEditPassword = (event, password) => {
    this.props.vaultStore.setCreatePassword(password);
  }

  onEditPasswordHint = (event, hint) => {
    this.props.vaultStore.setCreatePasswordHint(hint);
  }

  onEditPasswordRepeat = (event, password) => {
    this.props.vaultStore.setCreatePasswordRepeat(password);
  }

  onClickCreate = () => {
    const { createNameError, createPasswordRepeatError } = this.props.vaultStore;

    if (createNameError || createPasswordRepeatError) {
      return;
    }

    return this.props.vaultStore
      .createVault()
      .then(this.onClose)
      .catch((error) => {
        this.props.newError(error);
        this.onClose();
      });
  }

  onClose = () => {
    this.props.vaultStore.closeCreateModal();
  }
}

function mapDispatchToProps (dispatch) {
  return bindActionCreators({
    newError
  }, dispatch);
}

export default connect(
  null,
  mapDispatchToProps
)(VaultCreate);
