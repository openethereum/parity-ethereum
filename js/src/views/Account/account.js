import React, { Component, PropTypes } from 'react';
import { connect } from 'react-redux';
import { bindActionCreators } from 'redux';
import { FlatButton } from 'material-ui';
import ActionAccountBalance from 'material-ui/svg-icons/action/account-balance';
import ContentSend from 'material-ui/svg-icons/content/send';

import { FundAccount, Transfer } from '../../modals';
import { Actionbar, Page } from '../../ui';

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
        key='fundAccount'
        icon={ <ActionAccountBalance /> }
        label='fund account'
        primary
        onTouchTap={ this.onFundAccountClick } />
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
      <FundAccount
        address={ address }
        onClose={ this.onFundAccountClose } />
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

  onFundAccountClick = () => {
    this.setState({
      fundDialog: !this.state.fundDialog
    });
  }

  onFundAccountClose = () => {
    this.onFundAccountClick();
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
