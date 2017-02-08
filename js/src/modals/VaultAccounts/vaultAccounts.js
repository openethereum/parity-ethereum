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
import { Portal, SectionList } from '~/ui';

@observer
class VaultAccounts extends Component {
  static propTypes = {
    accounts: PropTypes.object.isRequired,
    newError: PropTypes.func.isRequired,
    vaultStore: PropTypes.object.isRequired
  }

  render () {
    const { accounts } = this.props;
    const { isModalAccountsOpen } = this.props.vaultStore;

    if (!isModalAccountsOpen) {
      return null;
    }

    const vaultAccounts = Object
      .keys(accounts)
      .filter((address) => accounts[address].uuid)
      .map((address) => accounts[address]);

    return (
      <Portal
        onClose={ this.onClose }
        open
        title={
          <FormattedMessage
            id='vaults.accounts.title'
            defaultMessage='Manage Vault Accounts'
          />
        }
      >
        <SectionList
          items={ vaultAccounts }
          renderItem={ this.renderAccount }
        />
      </Portal>
    );
  }

  renderAccount = (account) => {
    // const { vaultName } = this.props.vaultStore;

    return (
      <div>{ account.address }</div>
    );
  }

  onClose = () => {
    this.props.vaultStore.closeAccountsModal();
  }

  onExecute = () => {
    this.onClose();
  }
}

function mapStateToProps (state) {
  const { accounts } = state.personal;

  return { accounts };
}

function mapDispatchToProps (dispatch) {
  return bindActionCreators({
    newError
  }, dispatch);
}

export default connect(
  mapStateToProps,
  mapDispatchToProps
)(VaultAccounts);
