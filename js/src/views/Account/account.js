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
import { DeleteAccount, EditMeta, Faucet, PasswordManager, Shapeshift, Transfer, Verification } from '~/modals';
import { setVisibleAccounts } from '~/redux/providers/personalActions';
import { fetchCertifiers, fetchCertifications } from '~/redux/providers/certifications/actions';
import { Actionbar, Button, Page } from '~/ui';
import { DialIcon, VerifyIcon } from '~/ui/Icons';

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

    account: PropTypes.object,
    certifications: PropTypes.object,
    netVersion: PropTypes.string.isRequired,
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
    const { account } = this.props;
    const { address } = this.props.params;

    if (!account) {
      return null;
    }

    const isAvailable = !account.hardware || this.hwstore.isConnected(address);

    return (
      <div>
        { this.renderDeleteDialog(account) }
        { this.renderEditDialog(account) }
        { this.renderFaucetDialog() }
        { this.renderFundDialog() }
        { this.renderPasswordDialog(account) }
        { this.renderTransferDialog(account) }
        { this.renderVerificationDialog() }
        { this.renderActionbar(account) }
        <Page padded>
          <Header
            account={ account }
            disabled={ !isAvailable }
          />
          <Transactions
            address={ address }
          />
        </Page>
      </div>
    );
  }

  isKovan = (netVersion) => {
    return netVersion === '42';
  }

  isMainnet = (netVersion) => {
    return netVersion === '1';
  }

  isFaucettable = (netVersion, certifications, address) => {
    return this.isKovan(netVersion) || (
      this.isMainnet(netVersion) &&
      this.isSmsCertified(certifications, address)
    );
  }

  isSmsCertified = (_certifications, address) => {
    const certifications = _certifications && _certifications[address]
      ? _certifications[address].filter((cert) => cert.name.indexOf('smsverification') === 0)
      : [];

    return certifications.length !== 0;
  }

  renderActionbar (account) {
    const { certifications, netVersion } = this.props;
    const { address } = this.props.params;
    const isVerifiable = this.isMainnet(netVersion);
    const isFaucettable = this.isFaucettable(netVersion, certifications, address);

    const buttons = [
      <Button
        icon='send'
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
      isVerifiable
        ? (
          <Button
            icon={ <VerifyIcon /> }
            key='verification'
            label={
              <FormattedMessage
                id='account.button.verify'
                defaultMessage='verify'
              />
            }
            onClick={ this.store.toggleVerificationDialog }
          />
        )
        : null,
      isFaucettable
        ? (
          <Button
            icon={ <DialIcon /> }
            key='faucet'
            label={
              <FormattedMessage
                id='account.button.faucet'
                defaultMessage='Kovan ETH'
              />
            }
            onClick={ this.store.toggleFaucetDialog }
          />
        )
        : null,
      <Button
        icon='edit'
        key='editmeta'
        label={
          <FormattedMessage
            id='account.button.edit'
            defaultMessage='edit'
          />
        }
        onClick={ this.store.toggleEditDialog }
      />,
      !(account.external || account.hardware) && (
        <Button
          icon='unlock alternate'
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
        icon='trash'
        key='delete'
        label={
          account.external || account.hardware
            ? (
              <FormattedMessage
                id='account.button.forget'
                defaultMessage='forget'
              />
            )
            : (
              <FormattedMessage
                id='account.button.delete'
                defaultMessage='delete'
              />
            )
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

    if (account.external) {
      return (
        <DeleteAddress
          account={ account }
          confirmMessage={
            <FormattedMessage
              id='account.external.confirmDelete'
              defaultMessage='Are you sure you want to remove the following external address from your account list?'
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

  renderFaucetDialog () {
    const { netVersion } = this.props;

    if (!this.store.isFaucetVisible) {
      return null;
    }

    const { address } = this.props.params;

    return (
      <Faucet
        address={ address }
        netVersion={ netVersion }
        onClose={ this.store.toggleFaucetDialog }
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

  renderTransferDialog (account) {
    if (!this.store.isTransferVisible) {
      return null;
    }

    return (
      <Transfer
        account={ account }
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

function mapStateToProps (state, props) {
  const { address } = props.params;

  const { accounts } = state.personal;
  const certifications = state.certifications;
  const { netVersion } = state.nodeStatus;

  const account = (accounts || {})[address];

  return {
    account,
    certifications,
    netVersion
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
