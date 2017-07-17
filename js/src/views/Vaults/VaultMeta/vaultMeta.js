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
import { Button, Checkbox, Form, Input, Portal, VaultCard } from '@parity/ui';
import PasswordStrength from '@parity/ui/Form/PasswordStrength';
import { CheckIcon, CloseIcon } from '@parity/ui/Icons';

import styles from '../VaultCreate/vaultCreate.css';

@observer
class VaultMeta extends Component {
  static propTypes = {
    newError: PropTypes.func.isRequired,
    vaultStore: PropTypes.object.isRequired
  };

  state = {
    passwordEdit: false
  };

  render () {
    const { isBusyMeta, isModalMetaOpen, vault, vaultDescription, vaultPassword, vaultPasswordRepeat, vaultPasswordRepeatError, vaultPasswordOld, vaultPasswordHint } = this.props.vaultStore;
    const { passwordEdit } = this.state;

    if (!isModalMetaOpen) {
      return null;
    }

    return (
      <Portal
        busy={ isBusyMeta }
        buttons={ [
          <Button
            disabled={ isBusyMeta }
            icon={ <CloseIcon /> }
            key='close'
            label={
              <FormattedMessage
                id='vaults.editMeta.button.close'
                defaultMessage='close'
              />
            }
            onClick={ this.onClose }
          />,
          <Button
            disabled={ isBusyMeta || !!vaultPasswordRepeatError }
            icon={ <CheckIcon /> }
            key='vault'
            label={
              <FormattedMessage
                id='vaults.editMeta.button.save'
                defaultMessage='save'
              />
            }
            onClick={ this.onExecute }
          />
        ] }
        onClose={ this.onClose }
        open
        title={
          <FormattedMessage
            id='vaults.editMeta.title'
            defaultMessage='Edit Vault Metadata'
          />
        }
      >
        <VaultCard.Layout vault={ vault }>
          <Form>
            <div className={ styles.group }>
              <Input
                autoFocus
                hint={
                  <FormattedMessage
                    id='vaults.editMeta.description.hint'
                    defaultMessage='the description for this vault'
                  />
                }
                label={
                  <FormattedMessage
                    id='vaults.editMeta.description.label'
                    defaultMessage='vault description'
                  />
                }
                onChange={ this.onEditDescription }
                value={ vaultDescription }
              />
              <Input
                hint={
                  <FormattedMessage
                    id='vaults.editMeta.passwordHint.hint'
                    defaultMessage='your password hint for this vault'
                  />
                }
                label={
                  <FormattedMessage
                    id='vaults.editMeta.passwordHint.label'
                    defaultMessage='password hint'
                  />
                }
                onChange={ this.onEditPasswordHint }
                value={ vaultPasswordHint }
              />
            </div>
            <div className={ styles.group }>
              <Checkbox
                toggle
                checked={ passwordEdit }
                onClick={ this.onTogglePassword }
                label={
                  <FormattedMessage
                    id='vaults.editMeta.allowPassword'
                    defaultMessage='Change vault password'
                  />
                }
              />
              <div className={ [styles.passwords, passwordEdit ? null : styles.disabled].join(' ') }>
                <div className={ styles.password }>
                  <Input
                    disabled={ !passwordEdit }
                    hint={
                      <FormattedMessage
                        id='vaults.editMeta.currentPassword.hint'
                        defaultMessage='your current vault password'
                      />
                    }
                    label={
                      <FormattedMessage
                        id='vaults.editMeta.currentPassword.label'
                        defaultMessage='current password'
                      />
                    }
                    onChange={ this.onEditPasswordCurrent }
                    type='password'
                    value={ vaultPasswordOld }
                  />
                </div>
              </div>
              <div className={ [styles.passwords, passwordEdit ? null : styles.disabled].join(' ') }>
                <div className={ styles.password }>
                  <Input
                    disabled={ !passwordEdit }
                    hint={
                      <FormattedMessage
                        id='vaults.editMeta.password.hint'
                        defaultMessage='a strong, unique password'
                      />
                    }
                    label={
                      <FormattedMessage
                        id='vaults.editMeta.password.label'
                        defaultMessage='new password'
                      />
                    }
                    onChange={ this.onEditPassword }
                    type='password'
                    value={ vaultPassword }
                  />
                </div>
                <div className={ styles.password }>
                  <Input
                    disabled={ !passwordEdit }
                    error={ vaultPasswordRepeatError }
                    hint={
                      <FormattedMessage
                        id='vaults.editMeta.password2.hint'
                        defaultMessage='verify your new password'
                      />
                    }
                    label={
                      <FormattedMessage
                        id='vaults.editMeta.password2.label'
                        defaultMessage='new password (repeat)'
                      />
                    }
                    onChange={ this.onEditPasswordRepeat }
                    type='password'
                    value={ vaultPasswordRepeat }
                  />
                </div>
              </div>
              <div className={ passwordEdit ? null : styles.disabled }>
                <PasswordStrength input={ vaultPassword } />
              </div>
            </div>
          </Form>
        </VaultCard.Layout>
      </Portal>
    );

    // <InputChip
    //   addOnBlur
    //   hint={
    //     <FormattedMessage
    //       id='vaults.editMeta.tags.hint'
    //       defaultMessage='press <Enter> to add a tag'
    //     />
    //   }
    //   label={
    //     <FormattedMessage
    //       id='vaults.editMeta.tags.label'
    //       defaultMessage='(optional) tags'
    //     />
    //   }
    //   onTokensChange={ this.onEditTags }
    //   tokens={ vaultTags.slice() }
    // />
  }

  onEditDescription = (event, description) => {
    this.props.vaultStore.setVaultDescription(description);
  }

  onEditPasswordCurrent = (event, password) => {
    this.props.vaultStore.setVaultPasswordOld(password);
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

  onEditTags = (tags) => {
    this.props.vaultStore.setVaultTags(tags);
  }

  onTogglePassword = () => {
    this.setState({
      passwordEdit: !this.state.passwordEdit
    });
  }

  onExecute = () => {
    const { vaultPasswordRepeatError } = this.props.vaultStore;
    const { passwordEdit } = this.state;

    if (vaultPasswordRepeatError) {
      return;
    }

    return Promise
      .all([
        passwordEdit
          ? this.props.vaultStore.editVaultPassword()
          : true
      ])
      .then(() => {
        return this.props.vaultStore.editVaultMeta();
      })
      .catch(this.props.newError)
      .then(this.onClose);
  }

  onClose = () => {
    this.setState({
      passwordEdit: false
    });

    this.props.vaultStore.closeMetaModal();
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
)(VaultMeta);
