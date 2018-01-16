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
import { ConfirmDialog, VaultCard } from '~/ui';

import styles from '../VaultUnlock/vaultUnlock.css';

@observer
class VaultLock extends Component {
  static propTypes = {
    newError: PropTypes.func.isRequired,
    vaultStore: PropTypes.object.isRequired
  }

  render () {
    const { isBusyLock, isModalLockOpen, vault } = this.props.vaultStore;

    if (!isModalLockOpen) {
      return null;
    }

    return (
      <ConfirmDialog
        busy={ isBusyLock }
        disabledConfirm={ isBusyLock }
        disabledDeny={ isBusyLock }
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
        <VaultCard.Layout
          withBorder
          vault={ vault }
        />
      </ConfirmDialog>
    );
  }

  onExecute = () => {
    return this.props.vaultStore
      .closeVault()
      .catch(this.props.newError)
      .then(this.onClose);
  }

  onClose = () => {
    this.props.vaultStore.closeLockModal();
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
)(VaultLock);
