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

import { ConfirmDialog } from '~/ui';

import NameLayout from '../NameLayout';
import styles from '../vaults.css';

@observer
export default class ConfirmClose extends Component {
  static propTypes = {
    store: PropTypes.object.isRequired
  }

  render () {
    const { isBusyClose, isModalCloseOpen, vaultName } = this.props.store;

    if (!isModalCloseOpen) {
      return null;
    }

    return (
      <ConfirmDialog
        disabledConfim={ isBusyClose }
        onConfirm={ this.onExecute }
        onDeny={ this.onClose }
        open
        title={
          <FormattedMessage
            id='vaults.confirmClose.title'
            defaultMessage='Close Vault'
          />
        }
      >
        <div className={ styles.textbox }>
          <FormattedMessage
            id='vaults.confirmClose.info'
            defaultMessage="You are about to close a vault. Any accounts associated with the vault won't be visible after this operation concludes. To view the associated accounts, open the vault again."
          />
        </div>
        <NameLayout
          isOpen
          name={ vaultName }
        />
      </ConfirmDialog>
    );
  }

  onExecute = () => {
    this.onClose();
    return this.props.store.closeVault();
  }

  onClose = () => {
    this.props.store.closeCloseModal();
  }
}
