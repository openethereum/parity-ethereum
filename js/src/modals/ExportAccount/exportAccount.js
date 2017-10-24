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
import ExportInput from './exportInput';
import ExportStore from './exportStore';

@observer
class ExportAccount extends Component {
  static contextTypes = {
    api: PropTypes.object.isRequired
  };

  static propTypes = {
    accounts: PropTypes.object.isRequired,
    balances: PropTypes.object.isRequired,
    newError: PropTypes.func.isRequired,
    personalAccountsInfo: PropTypes.func.isRequired,
    onClose: PropTypes.func.isRequired
  };

  componentWillMount () {
    const { accounts, newError } = this.props;

    this.exportStore = new ExportStore(this.context.api, accounts, newError, null);
  }

  render () {
    const { canExport } = this.exportStore;

    return (
      <Portal
        buttons={ [
          <Button
            icon={ <CancelIcon /> }
            key='cancel'
            label={
              <FormattedMessage
                id='accounts.export.button.cancel'
                defaultMessage='Cancel'
              />
            }
            onClick={ this.onClose }
          />,
          <Button
            disabled={ !canExport }
            icon={ <CheckIcon /> }
            key='execute'
            label={
              <FormattedMessage
                id='accounts.export.button.export'
                defaultMessage='Export'
              />
            }
            onClick={ this.onExport }
          />
        ] }
        onClose={ this.onClose }
        open
        title={
          <FormattedMessage
            id='accounts.export.title'
            defaultMessage='Export an Account'
          />
        }
      >
        { this.renderList() }
      </Portal>
    );
  }

  renderList () {
    const { accounts } = this.props;

    const { selectedAccounts } = this.exportStore;

    const accountList = Object.values(accounts)
      .filter((account) => account.uuid)
      .map((account) => {
        account.checked = !!(selectedAccounts[account.address]);

        return account;
      });

    return (
      <SelectionList
        items={ accountList }
        noStretch
        onSelectClick={ this.onSelect }
        renderItem={ this.renderAccount }
      />
    );
  }

  renderAccount = (account) => {
    const { balances } = this.props;
    const balance = balances[account.address];
    const { changePassword, getPassword } = this.exportStore;
    const password = getPassword(account);

    return (
      <AccountCard
        account={ account }
        balance={ balance }
      >
        <div>
          <ExportInput
            account={ account }
            value={ password }
            onClick={ this.onClick }
            onChange={ changePassword }
          />
        </div>
      </AccountCard>
    );
  }

  onSelect = (account) => {
    this.exportStore.toggleSelectedAccount(account.address);
  }

  onClick = (address) => {
    this.exportStore.onClick(address);
  }

  onClose = () => {
    this.props.onClose();
  }

  onExport = () => {
    this.exportStore.onExport();
  }
}

function mapStateToProps (state) {
  const { balances } = state;
  const { accounts } = state.personal;

  return {
    accounts,
    balances
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
)(ExportAccount);
