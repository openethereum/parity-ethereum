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
import { ConfirmDialog, Input } from '~/ui';

import NameLayout from '../NameLayout';
import styles from '../vaults.css';

@observer
class ConfirmOpen extends Component {
  static propTypes = {
    newError: PropTypes.func.isRequired,
    vaultStore: PropTypes.object.isRequired
  }

  render () {
    const { isBusyOpen, isModalOpenOpen, vaultName, vaultPassword } = this.props.vaultStore;

    if (!isModalOpenOpen) {
      return null;
    }

    return (
      <ConfirmDialog
        busy={ isBusyOpen }
        disabledConfirm={ isBusyOpen }
        disabledDeny={ isBusyOpen }
        onConfirm={ this.onExecute }
        onDeny={ this.onClose }
        open
        title={
          <FormattedMessage
            id='vaults.confirmOpen.title'
            defaultMessage='Open Vault'
          />
        }
      >
        <div className={ styles.textbox }>
          <FormattedMessage
            id='vaults.confirmOpen.info'
            defaultMessage='You are about to open a vault. After confirming your password, all accounts associated with this vault will be visible. Closing the vault will remove the accounts from view until the vault is opened again.'
          />
        </div>
        <NameLayout
          isOpen
          name={ vaultName }
          withBorder
        />
        <Input
          hint={
            <FormattedMessage
              id='vaults.confirmOpen.password.hint'
              defaultMessage='the password specified when creating the vault'
            />
          }
          label={
            <FormattedMessage
              id='vaults.confirmOpen.password.label'
              defaultMessage='vault password'
            />
          }
          onChange={ this.onEditPassword }
          type='password'
          value={ vaultPassword }
        />
        <br />
      </ConfirmDialog>
    );
  }

  onEditPassword = (event, password) => {
    this.props.vaultStore.setVaultPassword(password);
  }

  onClose = () => {
    this.props.vaultStore.closeOpenModal();
  }

  onExecute = () => {
    return this.props.vaultStore
      .openVault()
      .catch(this.props.newError)
      .then(this.onClose);
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
)(ConfirmOpen);
