// Copyright 2015, 2016 Parity Technologies (UK) Ltd.
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

import React, { Component, PropTypes } from 'react';
import { connect } from 'react-redux';
import { bindActionCreators } from 'redux';
import ActionDelete from 'material-ui/svg-icons/action/delete';
import ContentCreate from 'material-ui/svg-icons/content/create';
import ContentSend from 'material-ui/svg-icons/content/send';
import LockIcon from 'material-ui/svg-icons/action/lock';
import VerifyIcon from 'material-ui/svg-icons/action/verified-user';

import { EditMeta, DeleteAccount, Shapeshift, Verification, Transfer, PasswordManager } from '~/modals';
import { Actionbar, Button, Page } from '~/ui';

import shapeshiftBtn from '~/../assets/images/shapeshift-btn.png';

import Header from './Header';
import Transactions from './Transactions';
import { setVisibleAccounts } from '~/redux/providers/personalActions';

import SMSVerificationStore from '~/modals/Verification/sms-store';
import EmailVerificationStore from '~/modals/Verification/email-store';

import styles from './account.css';

class Account extends Component {
  static contextTypes = {
    api: PropTypes.object.isRequired
  }

  static propTypes = {
    setVisibleAccounts: PropTypes.func.isRequired,
    images: PropTypes.object.isRequired,

    params: PropTypes.object,
    accounts: PropTypes.object,
    isTestnet: PropTypes.bool,
    balances: PropTypes.object
  }

  state = {
    showDeleteDialog: false,
    showEditDialog: false,
    showFundDialog: false,
    showVerificationDialog: false,
    verificationStore: null,
    showTransferDialog: false,
    showPasswordDialog: false
  }

  componentDidMount () {
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
    const { params, setVisibleAccounts } = props;
    const addresses = [ params.address ];
    setVisibleAccounts(addresses);
  }

  render () {
    const { accounts, balances } = this.props;
    const { address } = this.props.params;

    const account = (accounts || {})[address];
    const balance = (balances || {})[address];

    if (!account) {
      return null;
    }

    return (
      <div>
        { this.renderDeleteDialog(account) }
        { this.renderEditDialog(account) }
        { this.renderFundDialog() }
        { this.renderVerificationDialog() }
        { this.renderTransferDialog() }
        { this.renderPasswordDialog() }
        { this.renderActionbar() }
        <Page>
          <Header
            account={ account }
            balance={ balance }
          />
          <Transactions
            accounts={ accounts }
            address={ address } />
        </Page>
      </div>
    );
  }

  renderActionbar () {
    const { address } = this.props.params;
    const { balances } = this.props;
    const balance = balances[address];

    const showTransferButton = !!(balance && balance.tokens);

    const buttons = [
      <Button
        key='transferFunds'
        icon={ <ContentSend /> }
        label='transfer'
        disabled={ !showTransferButton }
        onClick={ this.onTransferClick } />,
      <Button
        key='shapeshift'
        icon={ <img src={ shapeshiftBtn } className={ styles.btnicon } /> }
        label='shapeshift'
        onClick={ this.onShapeshiftAccountClick } />,
      <Button
        key='sms-verification'
        icon={ <VerifyIcon /> }
        label='Verify'
        onClick={ this.openVerification } />,
      <Button
        key='editmeta'
        icon={ <ContentCreate /> }
        label='edit'
        onClick={ this.onEditClick } />,
      <Button
        key='passwordManager'
        icon={ <LockIcon /> }
        label='password'
        onClick={ this.onPasswordClick } />,
      <Button
        key='delete'
        icon={ <ActionDelete /> }
        label='delete account'
        onClick={ this.onDeleteClick } />
    ];

    return (
      <Actionbar
        title='Account Management'
        buttons={ buttons } />
    );
  }

  renderDeleteDialog (account) {
    const { showDeleteDialog } = this.state;

    if (!showDeleteDialog) {
      return null;
    }

    return (
      <DeleteAccount
        account={ account }
        onClose={ this.onDeleteClose } />
    );
  }

  renderEditDialog (account) {
    const { showEditDialog } = this.state;

    if (!showEditDialog) {
      return null;
    }

    return (
      <EditMeta
        account={ account }
        keys={ ['description', 'passwordHint'] }
        onClose={ this.onEditClick } />
    );
  }

  renderFundDialog () {
    const { showFundDialog } = this.state;

    if (!showFundDialog) {
      return null;
    }

    const { address } = this.props.params;

    return (
      <Shapeshift
        address={ address }
        onClose={ this.onShapeshiftAccountClose } />
    );
  }

  renderVerificationDialog () {
    if (!this.state.showVerificationDialog) {
      return null;
    }

    const store = this.state.verificationStore;
    const { address } = this.props.params;

    return (
      <Verification
        store={ store } account={ address }
        onSelectMethod={ this.selectVerificationMethod }
        onClose={ this.onVerificationClose }
      />
    );
  }

  renderTransferDialog () {
    const { showTransferDialog } = this.state;

    if (!showTransferDialog) {
      return null;
    }

    const { address } = this.props.params;
    const { accounts, balances, images } = this.props;
    const account = accounts[address];
    const balance = balances[address];

    return (
      <Transfer
        account={ account }
        balance={ balance }
        balances={ balances }
        images={ images }
        onClose={ this.onTransferClose } />
    );
  }

  renderPasswordDialog () {
    const { showPasswordDialog } = this.state;

    if (!showPasswordDialog) {
      return null;
    }

    const { address } = this.props.params;
    const { accounts } = this.props;
    const account = accounts[address];

    return (
      <PasswordManager
        account={ account }
        onClose={ this.onPasswordClose } />
    );
  }

  onDeleteClick = () => {
    this.setState({ showDeleteDialog: true });
  }

  onDeleteClose = () => {
    this.setState({ showDeleteDialog: false });
  }

  onEditClick = () => {
    this.setState({
      showEditDialog: !this.state.showEditDialog
    });
  }

  onShapeshiftAccountClick = () => {
    this.setState({
      showFundDialog: !this.state.showFundDialog
    });
  }

  onShapeshiftAccountClose = () => {
    this.onShapeshiftAccountClick();
  }

  openVerification = () => {
    this.setState({ showVerificationDialog: true });
  }

  selectVerificationMethod = (name) => {
    const { isTestnet } = this.props;
    if (typeof isTestnet !== 'boolean' || this.state.verificationStore) return;

    const { api } = this.context;
    const { address } = this.props.params;

    let verificationStore = null;
    if (name === 'sms') {
      verificationStore = new SMSVerificationStore(api, address, isTestnet);
    } else if (name === 'email') {
      verificationStore = new EmailVerificationStore(api, address, isTestnet);
    }
    this.setState({ verificationStore });
  }

  onVerificationClose = () => {
    this.setState({ showVerificationDialog: false });
  }

  onTransferClick = () => {
    this.setState({
      showTransferDialog: !this.state.showTransferDialog
    });
  }

  onTransferClose = () => {
    this.onTransferClick();
  }

  onPasswordClick = () => {
    this.setState({
      showPasswordDialog: !this.state.showPasswordDialog
    });
  }

  onPasswordClose = () => {
    this.onPasswordClick();
  }
}

function mapStateToProps (state) {
  const { accounts } = state.personal;
  const { isTest } = state.nodeStatus;
  const { balances } = state.balances;
  const { images } = state;

  return {
    accounts,
    isTestnet: isTest,
    balances,
    images
  };
}

function mapDispatchToProps (dispatch) {
  return bindActionCreators({
    setVisibleAccounts
  }, dispatch);
}

export default connect(
  mapStateToProps,
  mapDispatchToProps
)(Account);
