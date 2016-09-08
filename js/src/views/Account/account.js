import React, { Component, PropTypes } from 'react';
import { FlatButton } from 'material-ui';
import ActionAccountBalance from 'material-ui/svg-icons/action/account-balance';
import ContentSend from 'material-ui/svg-icons/content/send';

import { FundAccount, Transfer } from '../../modals';
import { Actionbar, Page } from '../../ui';

import Header from './Header';
import Transactions from './Transactions';

import styles from './account.css';

export default class Account extends Component {
  static contextTypes = {
    api: React.PropTypes.object,
    accounts: PropTypes.array,
    balances: PropTypes.object
  }

  static propTypes = {
    params: PropTypes.object
  }

  propName = null

  state = {
    fundDialog: false,
    transferDialog: false
  }

  render () {
    const { accounts } = this.context;
    const { address } = this.props.params;
    const account = accounts.find((_account) => _account.address === address);

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
            account={ account } />
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
    const account = this.context.accounts.find((_account) => _account.address === address);

    return (
      <Transfer
        account={ account }
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
