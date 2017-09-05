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
import { personalAccountsInfo } from '~/redux/providers/personalActions';
import { AccountCard, Button, Portal, SelectionList } from '~/ui';
import { CancelIcon, CheckIcon } from '~/ui/Icons';

@observer
class VaultAccounts extends Component {
  static contextTypes = {
    api: PropTypes.object.isRequired
  };

  static propTypes = {
    accounts: PropTypes.object.isRequired,
    newError: PropTypes.func.isRequired,
    personalAccountsInfo: PropTypes.func.isRequired,
    vaultStore: PropTypes.object.isRequired
  };

  render () {
    const { accounts } = this.props;
    const { isBusyAccounts, isModalAccountsOpen, selectedAccounts } = this.props.vaultStore;

    if (!isModalAccountsOpen) {
      return null;
    }

    const vaultAccounts = Object
      .keys(accounts)
      .filter((address) => accounts[address].uuid)
      .map((address) => accounts[address]);

    return (
      <Portal
        buttons={ [
          <Button
            disabled={ isBusyAccounts }
            icon={ <CancelIcon /> }
            key='cancel'
            label={
              <FormattedMessage
                id='vaults.accounts.button.cancel'
                defaultMessage='Cancel'
              />
            }
            onClick={ this.onClose }
          />,
          <Button
            disabled={ isBusyAccounts }
            icon={ <CheckIcon /> }
            key='execute'
            label={
              <FormattedMessage
                id='vaults.accounts.button.execute'
                defaultMessage='Set'
              />
            }
            onClick={ this.onExecute }
          />
        ] }
        busy={ isBusyAccounts }
        onClose={ this.onClose }
        open
        title={
          <FormattedMessage
            id='vaults.accounts.title'
            defaultMessage='Manage Vault Accounts'
          />
        }
      >
        { this.renderList(vaultAccounts, selectedAccounts) }
      </Portal>
    );
  }

  renderList (vaultAccounts) {
    return (
      <SelectionList
        isChecked={ this.isSelected }
        items={ vaultAccounts }
        noStretch
        onSelectClick={ this.onSelect }
        renderItem={ this.renderAccount }
      />
    );
  }

  renderAccount = (account) => {
    return (
      <AccountCard
        account={ account }
      />
    );
  }

  isSelected = (account) => {
    const { vaultName, selectedAccounts } = this.props.vaultStore;

    return account.meta.vault === vaultName
      ? !selectedAccounts[account.address]
      : selectedAccounts[account.address];
  }

  onSelect = (account) => {
    this.props.vaultStore.toggleSelectedAccount(account.address);
  }

  onClose = () => {
    this.props.vaultStore.closeAccountsModal();
  }

  onExecute = () => {
    const { api } = this.context;
    const { accounts, personalAccountsInfo, vaultStore } = this.props;
    const { vaultName, selectedAccounts } = this.props.vaultStore;

    const vaultAccounts = Object
      .keys(accounts)
      .filter((address) => accounts[address].uuid && selectedAccounts[address])
      .map((address) => accounts[address]);

    return vaultStore
      .moveAccounts(
        vaultName,
        vaultAccounts
          .filter((account) => account.meta.vault !== vaultName)
          .map((account) => account.address),
        vaultAccounts
          .filter((account) => account.meta.vault === vaultName)
          .map((account) => account.address)
      )
      .catch(this.props.newError)
      .then(() => {
        // TODO: We manually call parity_allAccountsInfo after all the promises
        // have been resolved. If bulk moves do become available in the future,
        // subscriptions can transparently take care of this instead of calling
        // and manually dispatching an update. (Using subscriptions currently
        // means allAccountsInfo is called after each and every move call)
        return api.parity
          .allAccountsInfo()
          .then(personalAccountsInfo);
      })
      .then(this.onClose);
  }
}

function mapStateToProps (state) {
  const { accounts } = state.personal;

  return {
    accounts
  };
}

function mapDispatchToProps (dispatch) {
  return bindActionCreators({
    newError,
    personalAccountsInfo
  }, dispatch);
}

export default connect(
  mapStateToProps,
  mapDispatchToProps
)(VaultAccounts);
