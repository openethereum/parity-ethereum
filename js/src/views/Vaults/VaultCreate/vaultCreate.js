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
import React, { Component } from 'react';
import PropTypes from 'prop-types';
import { FormattedMessage } from 'react-intl';
import { connect } from 'react-redux';
import { bindActionCreators } from 'redux';

import { newError } from '@parity/shared/redux/actions';
import { Button, Input, Portal } from '@parity/ui';
import PasswordStrength from '@parity/ui/Form/PasswordStrength';
import { CheckIcon, CloseIcon } from '@parity/ui/Icons';

import styles from './vaultCreate.css';

@observer
class VaultCreate extends Component {
  static propTypes = {
    newError: PropTypes.func.isRequired,
    vaultStore: PropTypes.object.isRequired
  }

  render () {
    const { isBusyCreate, isModalCreateOpen, vaultDescription, vaultName, vaultNameError, vaultPassword, vaultPasswordHint, vaultPasswordRepeat, vaultPasswordRepeatError } = this.props.vaultStore;
    const hasError = !!vaultNameError || !!vaultPasswordRepeatError;

    if (!isModalCreateOpen) {
      return null;
    }

    return (
      <Portal
        busy={ isBusyCreate }
        buttons={ [
          <Button
            disabled={ isBusyCreate }
            icon={ <CloseIcon /> }
            key='close'
            label={
              <FormattedMessage
                id='vaults.create.button.close'
                defaultMessage='close'
              />
            }
            onClick={ this.onClose }
          />,
          <Button
            disabled={ hasError || isBusyCreate }
            icon={ <CheckIcon /> }
            key='vault'
            label={
              <FormattedMessage
                id='vaults.create.button.vault'
                defaultMessage='create vault'
              />
            }
            onClick={ this.onCreate }
          />
        ] }
        onClose={ this.onClose }
        open
        title={
          <FormattedMessage
            id='vaults.create.title'
            defaultMessage='Create a new vault'
          />
        }
      >
        <div>
          <Input
            error={ vaultNameError }
            hint={
              <FormattedMessage
                id='vaults.create.name.hint'
                defaultMessage='a name for the vault'
              />
            }
            label={
              <FormattedMessage
                id='vaults.create.name.label'
                defaultMessage='vault name'
              />
            }
            onChange={ this.onEditName }
            value={ vaultName }
          />
          <Input
            hint={
              <FormattedMessage
                id='vaults.create.description.hint'
                defaultMessage='an extended description for the vault'
              />
            }
            label={
              <FormattedMessage
                id='vaults.create.descriptions.label'
                defaultMessage='(optional) description'
              />
            }
            onChange={ this.onEditDescription }
            value={ vaultDescription }
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
            value={ vaultPasswordHint }
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
                value={ vaultPassword }
              />
            </div>
            <div className={ styles.password }>
              <Input
                error={ vaultPasswordRepeatError }
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
                value={ vaultPasswordRepeat }
              />
            </div>
          </div>
          <PasswordStrength input={ vaultPassword } />
        </div>
      </Portal>
    );
  }

  onEditDescription = (event, description) => {
    this.props.vaultStore.setVaultDescription(description);
  }

  onEditName = (event, name) => {
    this.props.vaultStore.setVaultName(name);
  }

  onEditPassword = (event, password) => {
    this.props.vaultStore.setVaultPassword(password);
  }

  onEditPasswordHint = (event, hint) => {
    this.props.vaultStore.setVaultPasswordHint(hint);
  }

  onEditPasswordRepeat = (event, password) => {
    this.props.vaultStore.setVaultPasswordRepeat(password);
  }

  onCreate = () => {
    const { vaultNameError, vaultPasswordRepeatError } = this.props.vaultStore;

    if (vaultNameError || vaultPasswordRepeatError) {
      return;
    }

    return this.props.vaultStore
      .createVault()
      .catch(this.props.newError)
      .then(this.onClose);
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
