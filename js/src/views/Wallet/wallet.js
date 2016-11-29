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
import ActionDelete from 'material-ui/svg-icons/action/delete';
import ContentCreate from 'material-ui/svg-icons/content/create';
import ContentSend from 'material-ui/svg-icons/content/send';

import { EditMeta, DeleteAccount, Transfer } from '../../modals';
import { Actionbar, Button, Page } from '../../ui';

import Header from '../Account/Header';
import Transactions from '../Account/Transactions';
import { setVisibleAccounts } from '../../redux/providers/personalActions';

import styles from './wallet.css';

class Wallet extends Component {
  static contextTypes = {
    api: PropTypes.object.isRequired
  }

  static propTypes = {
    setVisibleAccounts: PropTypes.func.isRequired,
    images: PropTypes.object.isRequired,
    address: PropTypes.string.isRequired,
    wallets: PropTypes.object.isRequired,
    balances: PropTypes.object.isRequired
  }

  state = {
    showEditDialog: false,
    showTransferDialog: false
  }

  componentDidMount () {
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
    const { wallets, balances, address } = this.props;

    const wallet = (wallets || {})[address];
    const balance = (balances || {})[address];

    if (!wallet) {
      return null;
    }

    return (
      <div className={ styles.wallet }>
        { this.renderEditDialog(wallet) }
        { this.renderTransferDialog() }
        { this.renderActionbar() }
        <Page>
          <Header
            account={ wallet }
            balance={ balance } />
          <Transactions
            accounts={ wallets }
            address={ address } />
        </Page>
      </div>
    );
  }

  renderActionbar () {
    const { address, balances } = this.props;

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
        key='editmeta'
        icon={ <ContentCreate /> }
        label='edit'
        onClick={ this.onEditClick } />
    ];

    return (
      <Actionbar
        title='Wallet Management'
        buttons={ buttons } />
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
        keys={ ['description', 'passwordHint'] }
        onClose={ this.onEditClick } />
    );
  }

  renderTransferDialog () {
    const { showTransferDialog } = this.state;

    if (!showTransferDialog) {
      return null;
    }

    const { wallets, balances, images, address } = this.props;
    const wallet = wallets[address];
    const balance = balances[address];

    return (
      <Transfer
        account={ wallet }
        balance={ balance }
        balances={ balances }
        images={ images }
        onClose={ this.onTransferClose } />
    );
  }

  onEditClick = () => {
    this.setState({
      showEditDialog: !this.state.showEditDialog
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
}

function mapStateToProps (_, initProps) {
  const { address } = initProps.params;

  return (state) => {
    const { wallets } = state.personal;
    const { balances } = state.balances;
    const { images } = state;

    return {
      wallets,
      balances,
      images,
      address
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
