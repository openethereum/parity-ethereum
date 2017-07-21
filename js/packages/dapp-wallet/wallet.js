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

import React, { Component } from 'react';
import PropTypes from 'prop-types';
import { FormattedMessage } from 'react-intl';
import { connect } from 'react-redux';
import { bindActionCreators } from 'redux';
import moment from 'moment';

import HistoryStore from '@parity/shared/mobx/historyStore';
import { setVisibleAccounts } from '@parity/shared/redux/providers/personalActions';
import { nullableProptype } from '@parity/shared/util/proptypes';
import { Actionbar, Button, Page, Loading } from '@parity/ui';
import { DeleteIcon, EditIcon, SendIcon, SettingsIcon } from '@parity/ui/Icons';

import EditMeta from '@parity/dapp-account/EditMeta';
import Transfer from '@parity/dapp-account/Transfer';
import Delete from '@parity/dapp-address/Delete';
import Header from '@parity/dapp-account/Header';

import WalletDetails from './Details';
import WalletConfirmations from './Confirmations';
import WalletTransactions from './Transactions';
import WalletSettings from './WalletSettings';

import styles from './wallet.css';

const accountsHistory = HistoryStore.get('accounts');

class Wallet extends Component {
  static contextTypes = {
    api: PropTypes.object.isRequired
  };

  static propTypes = {
    address: PropTypes.string.isRequired,
    netVersion: PropTypes.string.isRequired,
    owned: PropTypes.bool.isRequired,
    setVisibleAccounts: PropTypes.func.isRequired,
    wallet: PropTypes.object.isRequired,
    walletAccount: nullableProptype(PropTypes.object.isRequired)
  };

  state = {
    showEditDialog: false,
    showSettingsDialog: false,
    showTransferDialog: false,
    showDeleteDialog: false
  };

  componentDidMount () {
    const { address } = this.props;

    accountsHistory.add(address, 'wallet');

    this.setVisibleAccounts();
  }

  componentWillReceiveProps (nextProps) {
    const prevAddress = this.props.address;
    const nextAddress = nextProps.address;

    if (prevAddress !== nextAddress) {
      this.setVisibleAccounts(nextProps);
    }
  }

  componentWillUnmount () {
    this.props.setVisibleAccounts([]);
  }

  setVisibleAccounts (props = this.props) {
    const { address, setVisibleAccounts } = props;
    const addresses = [ address ];

    setVisibleAccounts(addresses);
  }

  render () {
    const { walletAccount, wallet } = this.props;

    if (!walletAccount) {
      return null;
    }

    const { owners, require, dailylimit } = wallet;

    return (
      <div className={ styles.wallet }>
        { this.renderEditDialog(walletAccount) }
        { this.renderSettingsDialog() }
        { this.renderTransferDialog() }
        { this.renderDeleteDialog(walletAccount) }
        { this.renderActionbar() }
        <Page padded>
          <div className={ styles.info }>
            <Header
              className={ styles.header }
              account={ walletAccount }
              isContract
            >
              { this.renderInfos() }
            </Header>

            <WalletDetails
              className={ styles.details }
              owners={ owners }
              require={ require }
              dailylimit={ dailylimit }
            />
          </div>
          { this.renderDetails() }
        </Page>
      </div>
    );
  }

  renderInfos () {
    const { dailylimit } = this.props.wallet;
    const { api } = this.context;

    if (!dailylimit || !dailylimit.limit) {
      return null;
    }

    const _limit = api.util.fromWei(dailylimit.limit);

    if (_limit.equals(0)) {
      return null;
    }

    const limit = _limit.toFormat(3);
    const spent = api.util.fromWei(dailylimit.spent).toFormat(3);
    const date = moment(dailylimit.last.toNumber() * 24 * 3600 * 1000);

    return (
      <div>
        <br />
        <p>
          <FormattedMessage
            id='wallet.details.spent'
            defaultMessage='{spent} has been spent today, out of {limit} set as the daily limit, which has been reset on {date}'
            values={ {
              date: <span className={ styles.detail }>{ date.format('LL') }</span>,
              limit: <span className={ styles.detail }>{ limit }<span className={ styles.eth } /></span>,
              spent: <span className={ styles.detail }>{ spent }<span className={ styles.eth } /></span>
            } }
          />
        </p>
      </div>
    );
  }

  renderDetails () {
    const { address, netVersion, wallet } = this.props;
    const { owners, require, confirmations, transactions } = wallet;

    if (!owners || !require) {
      return (
        <div style={ { marginTop: '4em' } }>
          <Loading size='large' />
        </div>
      );
    }

    return [
      <WalletConfirmations
        address={ address }
        confirmations={ confirmations }
        netVersion={ netVersion }
        key='confirmations'
        owners={ owners }
        require={ require }
      />,

      <WalletTransactions
        address={ address }
        netVersion={ netVersion }
        key='transactions'
        transactions={ transactions }
      />
    ];
  }

  renderActionbar () {
    const { owned } = this.props;

    const buttons = [];

    if (owned) {
      buttons.push(
        <Button
          icon={ <SendIcon /> }
          key='transferFunds'
          label={
            <FormattedMessage
              id='wallet.buttons.transfer'
              defaultMessage='transfer'
            />
          }
          onClick={ this.onTransferClick }
        />
      );
    }

    buttons.push(
      <Button
        icon={ <DeleteIcon /> }
        key='delete'
        label={
          <FormattedMessage
            id='wallet.buttons.forget'
            defaultMessage='forget'
          />
        }
        onClick={ this.showDeleteDialog }
      />
    );

    buttons.push(
      <Button
        icon={ <EditIcon /> }
        key='editmeta'
        label={
          <FormattedMessage
            id='wallet.buttons.edit'
            defaultMessage='edit'
          />
        }
        onClick={ this.onEditClick }
      />
    );

    if (owned) {
      buttons.push(
        <Button
          icon={ <SettingsIcon /> }
          key='settings'
          label={
            <FormattedMessage
              id='wallet.buttons.settings'
              defaultMessage='settings'
            />
          }
          onClick={ this.onSettingsClick }
        />
      );
    }

    return (
      <Actionbar
        buttons={ buttons }
        title={
          <FormattedMessage
            id='wallet.title'
            defaultMessage='Wallet Management'
          />
        }
      />
    );
  }

  renderDeleteDialog (account) {
    const { showDeleteDialog } = this.state;

    return (
      <Delete
        account={ account }
        visible={ showDeleteDialog }
        route='/accounts'
        onClose={ this.closeDeleteDialog }
      />
    );
  }

  renderEditDialog (wallet) {
    const { showEditDialog } = this.state;

    if (!showEditDialog) {
      return null;
    }

    return (
      <EditMeta
        account={ wallet }
        keys={ ['description'] }
        onClose={ this.onEditClick }
      />
    );
  }

  renderSettingsDialog () {
    const { wallet } = this.props;
    const { showSettingsDialog } = this.state;

    if (!showSettingsDialog) {
      return null;
    }

    return (
      <WalletSettings
        wallet={ wallet }
        onClose={ this.onSettingsClick }
      />
    );
  }

  renderTransferDialog () {
    const { showTransferDialog } = this.state;

    if (!showTransferDialog) {
      return null;
    }

    const { walletAccount } = this.props;

    return (
      <Transfer
        account={ walletAccount }
        onClose={ this.onTransferClose }
      />
    );
  }

  onEditClick = () => {
    this.setState({
      showEditDialog: !this.state.showEditDialog
    });
  }

  onSettingsClick = () => {
    this.setState({
      showSettingsDialog: !this.state.showSettingsDialog
    });
  }

  onTransferClick = () => {
    this.setState({
      showTransferDialog: !this.state.showTransferDialog
    });
  }

  onTransferClose = () => {
    this.onTransferClick();
  }

  closeDeleteDialog = () => {
    this.setState({ showDeleteDialog: false });
  }

  showDeleteDialog = () => {
    this.setState({ showDeleteDialog: true });
  }
}

function mapStateToProps (_, initProps) {
  const { address } = initProps.params;

  return (state) => {
    const { netVersion } = state.nodeStatus;
    const { accountsInfo = {}, accounts = {} } = state.personal;
    const walletAccount = accounts[address] || accountsInfo[address] || null;

    if (walletAccount) {
      walletAccount.address = address;
    }

    const wallet = state.wallet.wallets[address] || {};
    const owned = !!accounts[address];

    return {
      address,
      netVersion,
      owned,
      wallet,
      walletAccount
    };
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
)(Wallet);
