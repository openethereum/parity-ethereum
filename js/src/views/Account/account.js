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

import shapeshiftBtn from '~/../assets/images/shapeshift-btn.png';
import HardwareStore from '~/mobx/hardwareStore';
import { EditMeta, DeleteAccount, Shapeshift, Verification, Transfer, PasswordManager } from '~/modals';
import { setVisibleAccounts } from '~/redux/providers/personalActions';
import { fetchCertifiers, fetchCertifications } from '~/redux/providers/certifications/actions';
import { Actionbar, Button, Page } from '~/ui';
import { DeleteIcon, EditIcon, LockedIcon, SendIcon, VerifyIcon } from '~/ui/Icons';

import DeleteAddress from '../Address/Delete';

import Header from './Header';
import Store from './store';
import Transactions from './Transactions';
import styles from './account.css';

@observer
class Account extends Component {
  static contextTypes = {
    api: PropTypes.object.isRequired
  };

  static propTypes = {
    fetchCertifiers: PropTypes.func.isRequired,
    fetchCertifications: PropTypes.func.isRequired,
    setVisibleAccounts: PropTypes.func.isRequired,

    accounts: PropTypes.object,
    balances: PropTypes.object,
    params: PropTypes.object
  }

  store = new Store();
  hwstore = HardwareStore.get(this.context.api);

  componentDidMount () {
    this.props.fetchCertifiers();
    this.setVisibleAccounts();
  }

  componentWillReceiveProps (nextProps) {
    const prevAddress = this.props.params.address;
    const nextAddress = nextProps.params.address;

    if (prevAddress !== nextAddress) {
      this.setVisibleAccounts(nextProps);
    }
  }

  componentWillUnmount () {
    this.props.setVisibleAccounts([]);
  }

  setVisibleAccounts (props = this.props) {
    const { params, setVisibleAccounts, fetchCertifications } = props;
    const addresses = [params.address];

    setVisibleAccounts(addresses);
    fetchCertifications(params.address);
  }

  render () {
    const { accounts, balances } = this.props;
    const { address } = this.props.params;

    const account = (accounts || {})[address];
    const balance = (balances || {})[address];

    if (!account) {
      return null;
    }

    const isAvailable = !account.hardware || this.hwstore.isConnected(address);

    return (
      <div>
        { this.renderDeleteDialog(account) }
        { this.renderEditDialog(account) }
        { this.renderFundDialog() }
        { this.renderPasswordDialog(account) }
        { this.renderTransferDialog(account, balance) }
        { this.renderVerificationDialog() }
        { this.renderActionbar(account, balance) }
        <Page padded>
          <Header
            account={ account }
            balance={ balance }
            disabled={ !isAvailable }
          />
          <Transactions
            accounts={ accounts }
            address={ address }
          />
        </Page>
      </div>
    );
  }

  renderActionbar (account, balance) {
    const showTransferButton = !!(balance && balance.tokens);

    const buttons = [
      <Button
        disabled={ !showTransferButton }
        icon={ <SendIcon /> }
        key='transferFunds'
        label={
          <FormattedMessage
            id='account.button.transfer'
            defaultMessage='transfer'
          />
        }
        onClick={ this.store.toggleTransferDialog }
      />,
      <Button
        icon={
          <img
            className={ styles.btnicon }
            src={ shapeshiftBtn }
          />
        }
        key='shapeshift'
        label={
          <FormattedMessage
            id='account.button.shapeshift'
            defaultMessage='shapeshift'
          />
        }
        onClick={ this.store.toggleFundDialog }
      />,
      <Button
        icon={ <VerifyIcon /> }
        key='sms-verification'
        label={
          <FormattedMessage
            id='account.button.verify'
            defaultMessage='verify'
          />
        }
        onClick={ this.store.toggleVerificationDialog }
      />,
      <Button
        icon={ <EditIcon /> }
        key='editmeta'
        label={
          <FormattedMessage
            id='account.button.edit'
            defaultMessage='edit'
          />
        }
        onClick={ this.store.toggleEditDialog }
      />,
      !account.hardware && (
        <Button
          icon={ <LockedIcon /> }
          key='passwordManager'
          label={
            <FormattedMessage
              id='account.button.password'
              defaultMessage='password'
            />
          }
          onClick={ this.store.togglePasswordDialog }
        />
      ),
      <Button
        icon={ <DeleteIcon /> }
        key='delete'
        label={
          <FormattedMessage
            id='account.button.delete'
            defaultMessage='delete'
          />
        }
        onClick={ this.store.toggleDeleteDialog }
      />
    ];

    return (
      <Actionbar
        buttons={ buttons }
        title={
          <FormattedMessage
            id='account.title'
            defaultMessage='Account Management'
          />
        }
      />
    );
  }

  renderDeleteDialog (account) {
    if (!this.store.isDeleteVisible) {
      return null;
    }

    if (account.hardware) {
      return (
        <DeleteAddress
          account={ account }
          confirmMessage={
            <FormattedMessage
              id='account.hardware.confirmDelete'
              defaultMessage='Are you sure you want to remove the following hardware address from your account list?'
            />
          }
          visible
          route='/accounts'
          onClose={ this.store.toggleDeleteDialog }
        />
      );
    }

    return (
      <DeleteAccount
        account={ account }
        onClose={ this.store.toggleDeleteDialog }
      />
    );
  }

  renderEditDialog (account) {
    if (!this.store.isEditVisible) {
      return null;
    }

    return (
      <EditMeta
        account={ account }
        onClose={ this.store.toggleEditDialog }
      />
    );
  }

  renderFundDialog () {
    if (!this.store.isFundVisible) {
      return null;
    }

    const { address } = this.props.params;

    return (
      <Shapeshift
        address={ address }
        onClose={ this.store.toggleFundDialog }
      />
    );
  }

  renderPasswordDialog (account) {
    if (!this.store.isPasswordVisible) {
      return null;
    }

    return (
      <PasswordManager
        account={ account }
        onClose={ this.store.togglePasswordDialog }
      />
    );
  }

  renderTransferDialog (account, balance) {
    if (!this.store.isTransferVisible) {
      return null;
    }

    const { balances } = this.props;

    return (
      <Transfer
        account={ account }
        balance={ balance }
        balances={ balances }
        onClose={ this.store.toggleTransferDialog }
      />
    );
  }

  renderVerificationDialog () {
    if (!this.store.isVerificationVisible) {
      return null;
    }

    const { address } = this.props.params;

    return (
      <Verification
        account={ address }
        onClose={ this.store.toggleVerificationDialog }
      />
    );
  }
}

function mapStateToProps (state) {
  const { accounts } = state.personal;
  const { balances } = state.balances;

  return {
    accounts,
    balances
  };
}

function mapDispatchToProps (dispatch) {
  return bindActionCreators({
    fetchCertifiers,
    fetchCertifications,
    setVisibleAccounts
  }, dispatch);
}

export default connect(
  mapStateToProps,
  mapDispatchToProps
)(Account);
