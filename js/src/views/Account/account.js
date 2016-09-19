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
import { FlatButton } from 'material-ui';
import ContentSend from 'material-ui/svg-icons/content/send';

import { Shapeshift, Transfer } from '../../modals';
import { Actionbar, Page } from '../../ui';

import shapeshiftBtn from '../../images/shapeshift-btn.png';

import Header from './Header';
import Transactions from './Transactions';

import styles from './account.css';

class Account extends Component {
  static propTypes = {
    params: PropTypes.object,
    accounts: PropTypes.object,
    balances: PropTypes.object,
    isTest: PropTypes.bool
  }

  propName = null

  state = {
    fundDialog: false,
    transferDialog: false
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
        { this.renderFundDialog() }
        { this.renderTransferDialog() }
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
    const buttons = [
      <FlatButton
        key='transferFunds'
        icon={ <ContentSend /> }
        label='transfer'
        primary
        onTouchTap={ this.onTransferClick } />,
      <FlatButton
        key='shapeshift'
        icon={ <img src={ shapeshiftBtn } className={ styles.btnicon } /> }
        label='shapeshift'
        primary
        onTouchTap={ this.onShapeshiftAccountClick } />
    ];

    return (
      <Actionbar
        title='Account Management'
        buttons={ buttons } />
    );
  }

  renderFundDialog () {
    const { fundDialog } = this.state;

    if (!fundDialog) {
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
    const { transferDialog } = this.state;

    if (!transferDialog) {
      return null;
    }

    const { address } = this.props.params;
    const { accounts, balances } = this.props;
    const account = accounts[address];
    const balance = balances[address];

    return (
      <Transfer
        account={ account }
        balance={ balance }
        balances={ balances }
        onClose={ this.onTransferClose } />
    );
  }

  onShapeshiftAccountClick = () => {
    this.setState({
      fundDialog: !this.state.fundDialog
    });
  }

  onShapeshiftAccountClose = () => {
    this.onShapeshiftAccountClick();
  }

  onTransferClick = () => {
    this.setState({
      transferDialog: !this.state.transferDialog
    });
  }

  onTransferClose = () => {
    this.onTransferClick();
  }
}

function mapStateToProps (state) {
  const { accounts } = state.personal;
  const { balances } = state.balances;
  const { isTest } = state.nodeStatus;

  return {
    isTest,
    accounts,
    balances
  };
}

function mapDispatchToProps (dispatch) {
  return bindActionCreators({}, dispatch);
}

export default connect(
  mapStateToProps,
  mapDispatchToProps
)(Account);
