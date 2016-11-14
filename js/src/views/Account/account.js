// Copyright 2015, 2016 Ethcore (UK) Ltd.
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
import ContentCreate from 'material-ui/svg-icons/content/create';
import ContentSend from 'material-ui/svg-icons/content/send';
import LockIcon from 'material-ui/svg-icons/action/lock';

import { EditMeta, Shapeshift, Transfer, PasswordManager } from '../../modals';
import { Actionbar, Button, Page } from '../../ui';

import shapeshiftBtn from '../../../assets/images/shapeshift-btn.png';

import Header from './Header';
import Transactions from './Transactions';

import styles from './account.css';

class Account extends Component {
  static propTypes = {
    params: PropTypes.object,
    accounts: PropTypes.object,
    balances: PropTypes.object,
    images: PropTypes.object.isRequired,
    isTest: PropTypes.bool
  }

  propName = null

  state = {
    showEditDialog: false,
    showFundDialog: false,
    showTransferDialog: false,
    showPasswordDialog: false
  }

  render () {
    const { accounts, balances, isTest } = this.props;
    const { address } = this.props.params;

    const account = (accounts || {})[address];
    const balance = (balances || {})[address];

    if (!account) {
      return null;
    }

    return (
      <div className={ styles.account }>
        { this.renderEditDialog(account) }
        { this.renderFundDialog() }
        { this.renderTransferDialog() }
        { this.renderPasswordDialog() }
        { this.renderActionbar() }
        <Page>
          <Header
            isTest={ isTest }
            account={ account }
            balance={ balance } />
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
        key='editmeta'
        icon={ <ContentCreate /> }
        label='edit'
        onClick={ this.onEditClick } />,
      <Button
        key='passwordManager'
        icon={ <LockIcon /> }
        label='password'
        onClick={ this.onPasswordClick } />
    ];

    return (
      <Actionbar
        title='Account Management'
        buttons={ buttons } />
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
  const { balances } = state.balances;
  const { images } = state;
  const { isTest } = state.nodeStatus;

  return {
    isTest,
    accounts,
    balances,
    images
  };
}

function mapDispatchToProps (dispatch) {
  return bindActionCreators({}, dispatch);
}

export default connect(
  mapStateToProps,
  mapDispatchToProps
)(Account);
